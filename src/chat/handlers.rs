use crate::auth::AuthenticatedUser;
use crate::{chat::CreatedMessage, db::Db};
use rocket::{http::Status, serde::json::Json};
use rocket_db_pools::Connection;

use super::{repo, StoredMessage};

#[rocket::post("/", data = "<body>")]
pub async fn insert_message(
    mut db: Connection<Db>,
    body: Json<CreatedMessage>,
    sender: AuthenticatedUser,
) -> Result<Json<StoredMessage>, Status> {
    let message = repo::insert_message(&mut db, sender.id, body.recipient_id, &body.content)
        .await
        .or(Err(Status::InternalServerError))?
        .ok_or(Status::Forbidden)?;

    Ok(Json(message))
}
