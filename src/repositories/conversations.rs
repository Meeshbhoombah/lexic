use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::conversation::{
    ConversationParticipant,
    ConversationSummary,
};

async fn add_participant(
    transaction: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    account_id: Uuid,
    role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO conversation_participants (
            conversation_id,
            account_id,
            role
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (conversation_id, account_id)
        DO NOTHING
        "#,
    )
    .bind(conversation_id)
    .bind(account_id)
    .bind(role)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn create_direct(
    db: &PgPool,
    created_by: Uuid,
    other_account_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let (low_id, high_id) = if created_by < other_account_id {
        (created_by, other_account_id)
    } else {
        (other_account_id, created_by)
    };

    let mut transaction = db.begin().await?;

    let conversation_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO conversations (
            id,
            kind,
            created_by,
            direct_account_low_id,
            direct_account_high_id
        )
        VALUES ($1, 'direct', $2, $3, $4)
        ON CONFLICT (
            direct_account_low_id,
            direct_account_high_id
        )
        DO NOTHING
        RETURNING id
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(created_by)
    .bind(low_id)
    .bind(high_id)
    .fetch_optional(&mut *transaction)
    .await?;

    let conversation_id = match conversation_id {
        Some(id) => id,
        None => {
            sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT id
                FROM conversations
                WHERE direct_account_low_id = $1
                  AND direct_account_high_id = $2
                "#,
            )
            .bind(low_id)
            .bind(high_id)
            .fetch_one(&mut *transaction)
            .await?
        }
    };

    add_participant(
        &mut transaction,
        conversation_id,
        low_id,
        "member",
    )
    .await?;

    add_participant(
        &mut transaction,
        conversation_id,
        high_id,
        "member",
    )
    .await?;

    transaction.commit().await?;

    Ok(conversation_id)
}

pub async fn create_group(
    db: &PgPool,
    created_by: Uuid,
    participant_ids: &[Uuid],
) -> Result<Uuid, sqlx::Error> {
    let mut transaction = db.begin().await?;
    let conversation_id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO conversations (
            id,
            kind,
            created_by
        )
        VALUES ($1, 'group', $2)
        "#,
    )
    .bind(conversation_id)
    .bind(created_by)
    .execute(&mut *transaction)
    .await?;

    add_participant(
        &mut transaction,
        conversation_id,
        created_by,
        "owner",
    )
    .await?;

    for account_id in participant_ids {
        if *account_id == created_by {
            continue;
        }

        add_participant(
            &mut transaction,
            conversation_id,
            *account_id,
            "member",
        )
        .await?;
    }

    transaction.commit().await?;

    Ok(conversation_id)
}

pub async fn list_for_account(
    db: &PgPool,
    account_id: Uuid,
) -> Result<Vec<ConversationSummary>, sqlx::Error> {
    sqlx::query_as::<_, ConversationSummary>(
        r#"
        SELECT
            c.id,
            c.kind,
            c.created_at,
            cp.last_read_at,
            (
                SELECT COUNT(*)
                FROM conversation_participants participant_count
                WHERE participant_count.conversation_id = c.id
            ) AS participant_count,
            (
                SELECT MAX(m.created_at)
                FROM messages m
                WHERE m.conversation_id = c.id
            ) AS last_message_at
        FROM conversations c
        INNER JOIN conversation_participants cp
            ON cp.conversation_id = c.id
        WHERE cp.account_id = $1
        ORDER BY
            COALESCE(
                (
                    SELECT MAX(m.created_at)
                    FROM messages m
                    WHERE m.conversation_id = c.id
                ),
                c.created_at
            ) DESC
        "#,
    )
    .bind(account_id)
    .fetch_all(db)
    .await
}

pub async fn list_participants(
    db: &PgPool,
    conversation_id: Uuid,
) -> Result<Vec<ConversationParticipant>, sqlx::Error> {
    sqlx::query_as::<_, ConversationParticipant>(
        r#"
        SELECT
            conversation_id,
            account_id,
            role,
            joined_at,
            last_read_at
        FROM conversation_participants
        WHERE conversation_id = $1
        ORDER BY joined_at ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(db)
    .await
}
