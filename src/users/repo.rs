use super::Chat;
use sqlx::{types::Uuid, PgConnection};

pub async fn invite_user(
    db: &mut PgConnection,
    sender_id: Uuid,
    recipient_id: Uuid,
    sender_public_key: &[u8],
) -> Result<Chat, sqlx::Error> {
    sqlx::query_as!(
        Chat,
        r#"
        INSERT INTO chats (sender_id, recipient_id, sender_public_key)
        VALUES ($1, $2, $3)
        RETURNING *;
        "#,
        sender_id,
        recipient_id,
        sender_public_key
    )
    .fetch_one(&mut *db)
    .await
}

pub async fn accept_user(
    db: &mut PgConnection,
    sender_id: Uuid,
    recipient_id: Uuid,
    recipient_public_key: &[u8],
) -> Result<Option<Chat>, sqlx::Error> {
    sqlx::query_as!(
        Chat,
        r#"
        UPDATE chats
        SET
            recipient_public_key = $3
        WHERE sender_id = $1 AND recipient_id = $2 AND recipient_public_key IS NULL
        RETURNING *;
        "#,
        sender_id,
        recipient_id,
        recipient_public_key
    )
    .fetch_optional(&mut *db)
    .await
}
