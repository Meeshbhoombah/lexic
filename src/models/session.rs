use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub account_id: Uuid,
    pub status: String,
    pub account_created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SessionRecord {
    pub account_id: Uuid,
    pub status: String,
    pub account_created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
