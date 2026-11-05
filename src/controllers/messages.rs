use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    controllers::auth,
    errors::AppError,
    models::message::MessageRecord,
    services::{
        self,
        messages::{CreateMessage, MessagePage},
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub client_message_id: Uuid,
    pub protocol_version: i16,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageHistoryQuery {
    pub before: Option<DateTime<Utc>>,
    pub before_id: Option<Uuid>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_account_id: Uuid,
    pub client_message_id: Uuid,
    pub protocol_version: i16,
    pub nonce: String,
    pub ciphertext: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessageHistoryResponse {
    pub messages: Vec<MessageResponse>,
    pub has_more: bool,
    pub next_before: Option<DateTime<Utc>>,
    pub next_before_id: Option<Uuid>,
}

impl From<MessageRecord> for MessageResponse {
    fn from(message: MessageRecord) -> Self {
        Self {
            id: message.id,
            conversation_id: message.conversation_id,
            sender_account_id: message.sender_account_id,
            client_message_id: message.client_message_id,
            protocol_version: message.protocol_version,
            nonce: URL_SAFE_NO_PAD.encode(message.nonce),
            ciphertext: URL_SAFE_NO_PAD.encode(message.ciphertext),
            created_at: message.created_at,
        }
    }
}

impl From<MessagePage> for MessageHistoryResponse {
    fn from(page: MessagePage) -> Self {
        Self {
            messages: page
                .messages
                .into_iter()
                .map(MessageResponse::from)
                .collect(),
            has_more: page.has_more,
            next_before: page.next_before,
            next_before_id: page.next_before_id,
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let account_id = auth::authenticated_account_id(&state, &headers).await?;

    let message = services::messages::create(
        &state.db,
        account_id,
        conversation_id,
        CreateMessage {
            client_message_id: request.client_message_id,
            protocol_version: request.protocol_version,
            nonce: request.nonce,
            ciphertext: request.ciphertext,
        },
    )
    .await?;

    Ok(Json(MessageResponse::from(message)))
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Query(query): Query<MessageHistoryQuery>,
) -> Result<Json<MessageHistoryResponse>, AppError> {
    let account_id = auth::authenticated_account_id(&state, &headers).await?;

    let page = services::messages::list(
        &state.db,
        account_id,
        conversation_id,
        query.before,
        query.before_id,
        query.limit,
    )
    .await?;

    Ok(Json(MessageHistoryResponse::from(page)))
}
