use crate::Validate;
use rocket::serde::{uuid::Uuid, Deserialize, Serialize};

pub mod handlers;
mod repo;

mod option_hex {
    use rocket::serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|s: String| {
            if s == "null" {
                return Ok(None);
            }

            hex::decode(s).map(|v| Some(v)).or(Err(D::Error::custom(
                "Failed to deserialize bytes from hex string",
            )))
        })?
    }

    pub fn serialize<S>(op: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let output = match op {
            Some(v) => hex::encode(v),
            None => "null".to_string(),
        };

        serializer.serialize_str(&output)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    id: i32,
    sender_id: Uuid,
    recipient_id: Uuid,

    #[serde(with = "hex::serde")]
    sender_public_key: Vec<u8>,

    #[serde(with = "option_hex")]
    recipient_public_key: Option<Vec<u8>>,
    created_at: sqlx::types::chrono::NaiveDateTime,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    #[serde(with = "hex::serde")]
    public_key: Vec<u8>,
}

impl Validate for PublicKey {
    fn validate(&self) -> bool {
        self.public_key.len() == 32
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: i32,
    username: String,
    created_at: sqlx::types::chrono::NaiveDateTime,
}
