use sqlx::PgPool;
use uuid::Uuid;

use crate::models::message::MessageRecord;

pub async fn create(
    db: &PgPool,
    conversation_id: Uuid,
    sender_account_id: Uuid,
    client_message_id: Uuid,
    protocol_version: i16,
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<MessageRecord, sqlx::Error> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        INSERT INTO messages (
            id,
            conversation_id,
            sender_account_id,
            client_message_id,
            protocol_version,
            nonce,
            ciphertext
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (
            conversation_id,
            sender_account_id,
            client_message_id
        )
        DO UPDATE SET
            client_message_id = EXCLUDED.client_message_id
        RETURNING
            id,
            conversation_id,
            sender_account_id,
            client_message_id,
            protocol_version,
            nonce,
            ciphertext,
            created_at
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(conversation_id)
    .bind(sender_account_id)
    .bind(client_message_id)
    .bind(protocol_version)
    .bind(nonce)
    .bind(ciphertext)
    .fetch_one(db)
    .await
}

pub async fn list_before(
    db: &PgPool,
    conversation_id: Uuid,
    before: Option<chrono::DateTime<chrono::Utc>>,
    before_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<MessageRecord>, sqlx::Error> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT
            id,
            conversation_id,
            sender_account_id,
            client_message_id,
            protocol_version,
            nonce,
            ciphertext,
            created_at
        FROM messages
        WHERE conversation_id = $1
          AND (
              $2::timestamptz IS NULL
              OR (created_at, id) < ($2, $3)
          )
        ORDER BY created_at DESC, id DESC
        LIMIT $4
        "#,
    )
    .bind(conversation_id)
    .bind(before)
    .bind(before_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
