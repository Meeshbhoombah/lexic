use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::AppError, models::message::MessageRecord, repositories::messages as message_repository,
};

const MAX_NONCE_BYTES: usize = 256;
const MAX_CIPHERTEXT_BYTES: usize = 1_048_576;
const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 100;

pub struct CreateMessage {
    pub client_message_id: Uuid,
    pub protocol_version: i16,
    pub nonce: String,
    pub ciphertext: String,
}

pub struct MessagePage {
    pub messages: Vec<MessageRecord>,
    pub has_more: bool,
    pub next_before: Option<DateTime<Utc>>,
    pub next_before_id: Option<Uuid>,
}

pub async fn create(
    db: &PgPool,
    authenticated_account_id: Uuid,
    conversation_id: Uuid,
    request: CreateMessage,
) -> Result<MessageRecord, AppError> {
    if request.protocol_version <= 0 {
        return Err(AppError::BadRequest);
    }

    let nonce = URL_SAFE_NO_PAD
        .decode(request.nonce.as_bytes())
        .map_err(|_| AppError::BadRequest)?;

    let ciphertext = URL_SAFE_NO_PAD
        .decode(request.ciphertext.as_bytes())
        .map_err(|_| AppError::BadRequest)?;

    if nonce.is_empty()
        || nonce.len() > MAX_NONCE_BYTES
        || ciphertext.is_empty()
        || ciphertext.len() > MAX_CIPHERTEXT_BYTES
    {
        return Err(AppError::BadRequest);
    }

    let is_participant = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM conversation_participants
            WHERE conversation_id = $1
              AND account_id = $2
        )
        "#,
    )
    .bind(conversation_id)
    .bind(authenticated_account_id)
    .fetch_one(db)
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "Failed to verify conversation membership");
        AppError::Internal
    })?;

    if !is_participant {
        return Err(AppError::Forbidden);
    }

    message_repository::create(
        db,
        conversation_id,
        authenticated_account_id,
        request.client_message_id,
        request.protocol_version,
        &nonce,
        &ciphertext,
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "Failed to persist message");
        AppError::Internal
    })
}

pub async fn list(
    db: &PgPool,
    authenticated_account_id: Uuid,
    conversation_id: Uuid,
    before: Option<DateTime<Utc>>,
    before_id: Option<Uuid>,
    requested_limit: Option<i64>,
) -> Result<MessagePage, AppError> {
    if before.is_some() != before_id.is_some() {
        return Err(AppError::BadRequest);
    }

    let is_participant = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM conversation_participants
            WHERE conversation_id = $1
              AND account_id = $2
        )
        "#,
    )
    .bind(conversation_id)
    .bind(authenticated_account_id)
    .fetch_one(db)
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "Failed to verify conversation membership");
        AppError::Internal
    })?;

    if !is_participant {
        return Err(AppError::Forbidden);
    }

    let limit = requested_limit
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);

    let mut messages =
        message_repository::list_before(db, conversation_id, before, before_id, limit + 1)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "Failed to retrieve message history");
                AppError::Internal
            })?;

    let has_more = messages.len() > limit as usize;

    if has_more {
        messages.pop();
    }

    messages.reverse();

    let (next_before, next_before_id) = if has_more {
        match messages.first() {
            Some(message) => (Some(message.created_at), Some(message.id)),
            None => (None, None),
        }
    } else {
        (None, None)
    };

    Ok(MessagePage {
        messages,
        has_more,
        next_before,
        next_before_id,
    })
}
