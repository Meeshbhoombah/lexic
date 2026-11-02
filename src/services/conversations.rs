use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::conversation::{ConversationParticipant, ConversationSummary},
    repositories::conversations as conversation_repository,
};

const MAX_GROUP_PARTICIPANTS: usize = 100;

async fn account_is_active(db: &PgPool, account_id: Uuid) -> Result<bool, AppError> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM accounts
            WHERE id = $1
              AND status = 'active'
        )
        "#,
    )
    .bind(account_id)
    .fetch_one(db)
    .await
    .map_err(|_| AppError::Internal)
}

pub async fn create_direct(
    db: &PgPool,
    authenticated_account_id: Uuid,
    other_account_id: Uuid,
) -> Result<Uuid, AppError> {
    if authenticated_account_id == other_account_id {
        return Err(AppError::BadRequest);
    }

    if !account_is_active(db, other_account_id).await? {
        return Err(AppError::BadRequest);
    }

    conversation_repository::create_direct(db, authenticated_account_id, other_account_id)
        .await
        .map_err(|_| AppError::Internal)
}

pub async fn create_group(
    db: &PgPool,
    authenticated_account_id: Uuid,
    participant_ids: &[Uuid],
) -> Result<Uuid, AppError> {
    if participant_ids.len() + 1 > MAX_GROUP_PARTICIPANTS {
        return Err(AppError::BadRequest);
    }

    let mut unique_participants = HashSet::new();

    for account_id in participant_ids {
        if *account_id == authenticated_account_id {
            continue;
        }

        if !unique_participants.insert(*account_id) {
            return Err(AppError::BadRequest);
        }

        if !account_is_active(db, *account_id).await? {
            return Err(AppError::BadRequest);
        }
    }

    let participants: Vec<Uuid> = unique_participants.into_iter().collect();

    conversation_repository::create_group(db, authenticated_account_id, &participants)
        .await
        .map_err(|_| AppError::Internal)
}

pub async fn list_for_account(
    db: &PgPool,
    authenticated_account_id: Uuid,
) -> Result<Vec<ConversationSummary>, AppError> {
    conversation_repository::list_for_account(db, authenticated_account_id)
        .await
        .map_err(|_| AppError::Internal)
}

pub async fn list_participants(
    db: &PgPool,
    authenticated_account_id: Uuid,
    conversation_id: Uuid,
) -> Result<Vec<ConversationParticipant>, AppError> {
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
    .map_err(|_| AppError::Internal)?;

    if !is_participant {
        return Err(AppError::Forbidden);
    }

    conversation_repository::list_participants(db, conversation_id)
        .await
        .map_err(|_| AppError::Internal)
}
