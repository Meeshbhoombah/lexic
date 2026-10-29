use axum::{Router, routing::post};

use crate::{controllers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/v1/session",
        post(controllers::sessions::get_or_create),
    )
}
