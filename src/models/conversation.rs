use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub kind: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ConversationParticipant {
    pub conversation_id: Uuid,
    pub account_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub kind: String,
    pub created_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub participant_count: i64,
    pub last_message_at: Option<DateTime<Utc>>,
}
