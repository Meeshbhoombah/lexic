use axum::http::{HeaderMap, header};
use uuid::Uuid;

use crate::{errors::AppError, services, state::AppState};

const COOKIE_NAME: &str = "lexic_session";

fn read_session_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            (name == COOKIE_NAME).then_some(value)
        })
}

pub async fn authenticated_account_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Uuid, AppError> {
    services::sessions::authenticate(
        &state.db,
        read_session_token(headers),
    )
    .await
}
