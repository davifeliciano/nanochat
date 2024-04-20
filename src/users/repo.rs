use super::{handlers::UserFilter, Chat, User};
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

pub async fn filtered_search_users(
    db: &mut PgConnection,
    user_id: Uuid,
    q: &str,
    filter: UserFilter,
) -> Result<Vec<User>, sqlx::Error> {
    match filter {
        UserFilter::Invited => {
            sqlx::query_as!(
                User,
                r#"
                SELECT u.id, u.username, u.created_at
                FROM users u
                JOIN chats c ON c.recipient_id = u.id
                WHERE c.sender_id = $1 AND u.username %> $2;
                "#,
                user_id,
                q
            )
            .fetch_all(&mut *db)
            .await
        }
        UserFilter::Pending => {
            sqlx::query_as!(
                User,
                r#"
                SELECT u.id, u.username, u.created_at
                FROM users u
                JOIN chats c ON c.sender_id = u.id
                WHERE c.recipient_id = $1 AND c.recipient_public_key IS NULL AND u.username %> $2;
                "#,
                user_id,
                q
            )
            .fetch_all(&mut *db)
            .await
        }
        UserFilter::Friends => {
            sqlx::query_as!(
                User,
                r#"
                SELECT u.id, u.username, u.created_at
                FROM users u
                JOIN chats c ON c.sender_id = u.id OR c.recipient_id = u.id
                WHERE u.id <> $1 AND c.recipient_public_key IS NOT NULL AND u.username %> $2;
                "#,
                user_id,
                q
            )
            .fetch_all(&mut *db)
            .await
        }
    }
}

pub async fn filtered_get_users(
    db: &mut PgConnection,
    user_id: Uuid,
    filter: UserFilter,
) -> Result<Vec<User>, sqlx::Error> {
    match filter {
        UserFilter::Invited => {
            sqlx::query_as!(
                User,
                r#"
                SELECT u.id, u.username, u.created_at
                FROM users u
                JOIN chats c ON c.recipient_id = u.id
                WHERE c.sender_id = $1;
                "#,
                user_id,
            )
            .fetch_all(&mut *db)
            .await
        }
        UserFilter::Pending => {
            sqlx::query_as!(
                User,
                r#"
                SELECT u.id, u.username, u.created_at
                FROM users u
                JOIN chats c ON c.sender_id = u.id
                WHERE c.recipient_id = $1 AND c.recipient_public_key IS NULL;
                "#,
                user_id,
            )
            .fetch_all(&mut *db)
            .await
        }
        UserFilter::Friends => {
            sqlx::query_as!(
                User,
                r#"
                SELECT u.id, u.username, u.created_at
                FROM users u
                JOIN chats c ON c.sender_id = u.id OR c.recipient_id = u.id
                WHERE u.id <> $1 AND c.recipient_public_key IS NOT NULL;
                "#,
                user_id,
            )
            .fetch_all(&mut *db)
            .await
        }
    }
}

pub async fn search_users(
    db: &mut PgConnection,
    user_id: Uuid,
    q: &str,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, username, created_at
        FROM users
        WHERE id <> $1 AND username %> $2;
        "#,
        user_id,
        q,
    )
    .fetch_all(&mut *db)
    .await
}
