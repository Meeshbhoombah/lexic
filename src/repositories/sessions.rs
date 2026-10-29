use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        account::AccountRecord,
        session::{SessionRecord, SessionResponse},
    },
};

pub async fn find_active(
    db: &PgPool,
    token_hash: &[u8],
) -> Result<Option<SessionResponse>, AppError> {
    let session = sqlx::query_as::<_, SessionRecord>(
        r#"
        SELECT
            a.id AS account_id,
            a.status,
            a.created_at AS account_created_at,
            s.expires_at
        FROM sessions AS s
        INNER JOIN accounts AS a ON a.id = s.account_id
        WHERE s.token_hash = $1
          AND s.expires_at > NOW()
          AND a.status = 'active'
        "#,
    )
    .bind(token_hash)
    .fetch_optional(db)
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "Failed to look up session");
        AppError::Internal
    })?;

    Ok(session.map(|record| SessionResponse {
        account_id: record.account_id,
        status: record.status,
        account_created_at: record.account_created_at,
        expires_at: record.expires_at,
    }))
}

pub async fn create(
    db: &PgPool,
    token_hash: &[u8],
) -> Result<SessionResponse, AppError> {
    let mut tx = db.begin().await.map_err(|error| {
        tracing::error!(error = %error, "Failed to begin session transaction");
        AppError::Internal
    })?;

    let account = sqlx::query_as::<_, AccountRecord>(
        r#"
        INSERT INTO accounts (id)
        VALUES ($1)
        RETURNING id, status, created_at
        "#,
    )
    .bind(Uuid::now_v7())
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "Failed to create account");
        AppError::Internal
    })?;

    let expires_at = Utc::now() + Duration::days(30);

    sqlx::query(
        r#"
        INSERT INTO sessions (id, account_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(account.id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(&mut *tx)
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "Failed to create session");
        AppError::Internal
    })?;

    tx.commit().await.map_err(|error| {
        tracing::error!(error = %error, "Failed to commit session");
        AppError::Internal
    })?;

    Ok(SessionResponse {
        account_id: account.id,
        status: account.status,
        account_created_at: account.created_at,
        expires_at,
    })
}
