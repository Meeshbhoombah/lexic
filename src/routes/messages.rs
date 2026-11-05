use axum::{
    Router,
    routing::{get, post},
};

use crate::{controllers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/v1/conversations/{id}/messages",
        get(controllers::messages::list).post(controllers::messages::create),
    )
}
