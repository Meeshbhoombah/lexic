use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{controllers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/accounts", post(controllers::accounts::create))
        .route(
            "/api/v1/accounts/me/encryption-key",
            put(controllers::accounts::put_encryption_key),
        )
        .route(
            "/api/v1/accounts/{id}/encryption-key",
            get(controllers::accounts::get_encryption_key),
        )
}
