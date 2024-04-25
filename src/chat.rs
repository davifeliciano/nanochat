use rocket::serde::Serialize;
use sqlx::types::Uuid;

pub mod repo;

include!(concat!(env!("OUT_DIR"), "/chat.rs"));

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct StoredMessage {
    pub id: i32,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,

    #[serde(with = "hex::serde")]
    pub content: Vec<u8>,
    pub created_at: sqlx::types::chrono::NaiveDateTime,
}
