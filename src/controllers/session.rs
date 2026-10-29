use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
};

use crate::{errors::AppError, services, state::AppState};

const COOKIE_NAME: &str = "lexic_session";

fn read_cookie(headers: &HeaderMap) -> Option<&str> {
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

pub async fn get_or_create(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let result = services::sessions::establish(
        &state.db,
        read_cookie(&headers),
    )
    .await?;

    let max_age = (result.session.expires_at - chrono::Utc::now())
        .num_seconds()
        .max(0);

    let mut cookie = format!(
        "{COOKIE_NAME}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}",
        result.token
    );

    if state.cookie_secure {
        cookie.push_str("; Secure");
    }

    let cookie = cookie.parse::<HeaderValue>().map_err(|error| {
        tracing::error!(error = %error, "Failed to construct session cookie");
        AppError::Internal
    })?;

    let mut response = Json(result.session).into_response();

    response.headers_mut().insert(header::SET_COOKIE, cookie);
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );

    Ok(response)
}
