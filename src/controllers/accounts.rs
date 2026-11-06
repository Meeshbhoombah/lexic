use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::account::{CreateAccountRequest, CreateAccountResponse},
    services,
    state::AppState,
};

#[derive(Deserialize)]
pub struct EncryptionKeyRequest {
    pub public_key: String,
}

#[derive(Serialize)]
pub struct EncryptionKeyResponse {
    pub account_id: Uuid,
    pub public_key: String,
    pub key_version: i16,
}

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<Json<CreateAccountResponse>, AppError> {
    let account = services::accounts::create(&state.db, &request.credential).await?;

    Ok(Json(account))
}

pub async fn put_encryption_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<EncryptionKeyRequest>,
) -> Result<Json<EncryptionKeyResponse>, AppError> {
    let account_id = crate::controllers::auth::authenticated_account_id(&state, &headers).await?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(request.public_key.as_bytes())
        .map_err(|_| AppError::BadRequest)?;
    if bytes.len() != 65 {
        return Err(AppError::BadRequest);
    }
    sqlx::query("INSERT INTO account_encryption_keys (account_id, public_key) VALUES ($1, $2) ON CONFLICT (account_id) DO UPDATE SET public_key = EXCLUDED.public_key, updated_at = NOW()")
        .bind(account_id).bind(&bytes).execute(&state.db).await.map_err(|error| { tracing::error!(error = %error, "Failed to save encryption key"); AppError::Internal })?;
    Ok(Json(EncryptionKeyResponse {
        account_id,
        public_key: request.public_key,
        key_version: 1,
    }))
}

pub async fn get_encryption_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<Uuid>,
) -> Result<Json<EncryptionKeyResponse>, AppError> {
    crate::controllers::auth::authenticated_account_id(&state, &headers).await?;
    let row: Option<(Vec<u8>, i16)> = sqlx::query_as(
        "SELECT public_key, key_version FROM account_encryption_keys WHERE account_id = $1",
    )
    .bind(target_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::Internal)?;
    let (key, key_version) = row.ok_or(AppError::BadRequest)?;
    Ok(Json(EncryptionKeyResponse {
        account_id: target_id,
        public_key: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(key),
        key_version,
    }))
}
