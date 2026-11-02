use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    controllers::auth,
    errors::AppError,
    models::conversation::{
        ConversationParticipant,
        ConversationSummary,
    },
    services,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateDirectRequest {
    pub other_account_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub participant_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreateConversationResponse {
    pub conversation_id: Uuid,
}

pub async fn create_direct(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateDirectRequest>,
) -> Result<Json<CreateConversationResponse>, AppError> {
    let account_id =
        auth::authenticated_account_id(&state, &headers).await?;

    let conversation_id = services::conversations::create_direct(
        &state.db,
        account_id,
        request.other_account_id,
    )
    .await?;

    Ok(Json(CreateConversationResponse {
        conversation_id,
    }))
}

pub async fn create_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateGroupRequest>,
) -> Result<Json<CreateConversationResponse>, AppError> {
    let account_id =
        auth::authenticated_account_id(&state, &headers).await?;

    let conversation_id = services::conversations::create_group(
        &state.db,
        account_id,
        &request.participant_ids,
    )
    .await?;

    Ok(Json(CreateConversationResponse {
        conversation_id,
    }))
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ConversationSummary>>, AppError> {
    let account_id =
        auth::authenticated_account_id(&state, &headers).await?;

    let conversations = services::conversations::list_for_account(
        &state.db,
        account_id,
    )
    .await?;

    Ok(Json(conversations))
}

pub async fn list_participants(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Vec<ConversationParticipant>>, AppError> {
    let account_id =
        auth::authenticated_account_id(&state, &headers).await?;

    let participants = services::conversations::list_participants(
        &state.db,
        account_id,
        conversation_id,
    )
    .await?;

    Ok(Json(participants))
}
