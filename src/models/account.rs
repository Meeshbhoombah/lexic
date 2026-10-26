use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub credential: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAccountResponse {
    pub account_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountRecord {
    pub id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
