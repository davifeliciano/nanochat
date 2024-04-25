use super::{Chat, PublicKey, User};
use crate::{auth::AuthenticatedUser, chat::StoredMessage, db::Db, users::repo, Validate};
use rocket::{http::Status, serde::json::Json, FromForm, FromFormField};
use rocket_db_pools::Connection;
use sqlx::types::Uuid;

#[rocket::post("/<recipient_id>/invite", data = "<body>")]
pub async fn invite(
    mut db: Connection<Db>,
    body: Json<PublicKey>,
    recipient_id: Uuid,
    sender: AuthenticatedUser,
) -> Result<Json<Chat>, Status> {
    if !body.validate() {
        return Err(Status::UnprocessableEntity);
    }

    let chat = repo::invite_user(&mut db, sender.id, recipient_id, &body.public_key)
        .await
        .map_err(|e| match e.as_database_error() {
            Some(e) if e.is_unique_violation() => Status::Conflict,
            Some(e) if e.is_foreign_key_violation() => Status::NotFound,
            Some(e) if e.is_check_violation() => Status::UnprocessableEntity,
            _ => Status::InternalServerError,
        })?;

    Ok(Json(chat))
}

#[rocket::post("/<sender_id>/accept", data = "<body>")]
pub async fn accept(
    mut db: Connection<Db>,
    body: Json<PublicKey>,
    sender_id: Uuid,
    recipient: AuthenticatedUser,
) -> Result<Json<Chat>, Status> {
    if !body.validate() {
        return Err(Status::UnprocessableEntity);
    }

    let chat = repo::accept_user(&mut db, sender_id, recipient.id, &body.public_key)
        .await
        .map_err(|e| match e.as_database_error() {
            Some(e) if e.is_check_violation() => Status::UnprocessableEntity,
            _ => Status::InternalServerError,
        })?
        .ok_or(Status::NotFound)?;

    Ok(Json(chat))
}

#[derive(FromFormField)]
pub enum UserFilter {
    Invited,
    Pending,
    Friends,
}

#[rocket::get("/?<q>&<filter>", rank = 2)]
pub async fn filtered_search(
    mut db: Connection<Db>,
    user: AuthenticatedUser,
    q: Option<&str>,
    filter: UserFilter,
) -> Result<Json<Vec<User>>, Status> {
    let users = match q {
        Some(s) => repo::filtered_search_users(&mut db, user.id, s, filter).await,
        None => repo::filtered_get_users(&mut db, user.id, filter).await,
    }
    .or(Err(Status::InternalServerError))?;

    Ok(Json(users))
}

#[rocket::get("/?<q>", rank = 3)]
pub async fn search(
    mut db: Connection<Db>,
    user: AuthenticatedUser,
    q: &str,
) -> Result<Json<Vec<User>>, Status> {
    let users = repo::search_users(&mut db, user.id, q)
        .await
        .or(Err(Status::InternalServerError))?;

    Ok(Json(users))
}

#[derive(FromForm)]
pub struct MessagePage {
    pub start_timestamp: i64,
    pub start_id: i32,
    pub limit: i64,
}

#[rocket::get("/<friend_id>/messages?<page..>")]
pub async fn get_message_page(
    mut db: Connection<Db>,
    user: AuthenticatedUser,
    friend_id: Uuid,
    page: MessagePage,
) -> Result<Json<Vec<StoredMessage>>, Status> {
    let start_timestamp = chrono::DateTime::from_timestamp(page.start_timestamp, 0)
        .ok_or(Status::UnprocessableEntity)?
        .naive_utc();

    let messages = crate::chat::repo::get_message_page(
        &mut db,
        user.id,
        friend_id,
        start_timestamp,
        page.start_id,
        page.limit,
    )
    .await
    .or(Err(Status::InternalServerError))?;

    Ok(Json(messages))
}
