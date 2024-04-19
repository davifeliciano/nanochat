use super::{Chat, PublicKey};
use crate::{auth::AuthenticatedUser, db::Db, users::repo, Validate};
use rocket::{http::Status, serde::json::Json};
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
