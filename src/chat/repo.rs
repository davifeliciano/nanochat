use super::StoredMessage;
use sqlx::{
    types::{chrono::NaiveDateTime, Uuid},
    PgConnection,
};

pub async fn get_message_page(
    db: &mut PgConnection,
    sender_id: Uuid,
    recipient_id: Uuid,
    start_timestamp: NaiveDateTime,
    start_id: i32,
    limit: i64,
) -> Result<Vec<StoredMessage>, sqlx::Error> {
    sqlx::query_as!(
        StoredMessage,
        r#"
        SELECT * FROM messages
        WHERE
            (sender_id = $1 AND recipient_id = $2) OR
            (sender_id = $2 AND recipient_id = $1) AND
            (created_at, id) < ($3, $4)
        LIMIT $5;
        "#,
        sender_id,
        recipient_id,
        start_timestamp,
        start_id,
        limit
    )
    .fetch_all(&mut *db)
    .await
}

pub async fn insert_message(
    db: &mut PgConnection,
    sender_id: Uuid,
    recipient_id: Uuid,
    content: &Vec<u8>,
) -> Result<Option<StoredMessage>, sqlx::Error> {
    sqlx::query_as!(
        StoredMessage,
        r#"
        INSERT INTO messages
            (sender_id, recipient_id, content)
        SELECT
            $1, $2, $3
        WHERE exists(
            SELECT 1 FROM chats
            WHERE sender_id = $1 AND recipient_id = $2
                AND recipient_public_key IS NOT NULL
        )
        RETURNING *;
        "#,
        sender_id,
        recipient_id,
        content,
    )
    .fetch_optional(db)
    .await
}
