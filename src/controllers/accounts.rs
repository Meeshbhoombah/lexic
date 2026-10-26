use axum::{Json, extract::State};

use crate::{
    errors::AppError,
    models::account::{CreateAccountRequest, CreateAccountResponse},
    services,
    state::AppState,
};

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<Json<CreateAccountResponse>, AppError> {
    let account = services::accounts::create(&state.db, &request.credential).await?;

    Ok(Json(account))
}
