use super::{Message, StoredMessage};
use sqlx::{
    postgres::PgQueryResult,
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
    msg: &Message,
) -> Result<PgQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO messages
            (sender_id, recipient_id, content, created_at)
        VALUES
            ($1, $2, $3, $4);
        "#,
        Uuid::parse_str(&msg.sender_id).unwrap(),
        Uuid::parse_str(&msg.recipient_id).unwrap(),
        msg.content,
        chrono::DateTime::from_timestamp(msg.created_at, 0)
            .unwrap()
            .naive_utc()
    )
    .execute(&mut *db)
    .await
}
