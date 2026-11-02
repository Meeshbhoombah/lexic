use axum::{
    Router,
    routing::{get, post},
};

use crate::{controllers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/conversations",
            get(controllers::conversations::list).post(controllers::conversations::create_direct),
        )
        .route(
            "/api/v1/conversations/group",
            post(controllers::conversations::create_group),
        )
        .route(
            "/api/v1/conversations/{id}/participants",
            get(controllers::conversations::list_participants),
        )
}
