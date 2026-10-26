use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::account::{AccountRecord, CreateAccountResponse},
};

pub async fn create(db: &PgPool) -> Result<CreateAccountResponse, AppError> {
    let mut transaction = db.begin().await.map_err(|error| {
        tracing::error!(
            error = %error,
            "Failed to begin account transaction"
        );
        AppError::Internal
    })?;

    let account_id = Uuid::now_v7();

    let account = sqlx::query_as::<_, AccountRecord>(
        r#"
        INSERT INTO accounts (id)
        VALUES ($1)
        RETURNING id, status, created_at
        "#,
    )
    .bind(account_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!(
            error = %error,
            "Failed to insert account"
        );
        AppError::Internal
    })?;

    sqlx::query(
        r#"
        INSERT INTO account_connections (id, account_id, provider)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(account.id)
    .bind("lexe")
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        tracing::error!(
            error = %error,
            "Failed to insert account connection"
        );
        AppError::Internal
    })?;

    transaction.commit().await.map_err(|error| {
        tracing::error!(
            error = %error,
            "Failed to commit account transaction"
        );
        AppError::Internal
    })?;

    Ok(CreateAccountResponse {
        account_id: account.id,
        status: account.status,
        created_at: account.created_at,
    })
}
