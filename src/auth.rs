use crate::config::Config;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    serde::{uuid::Uuid, Deserialize, Serialize},
    Request,
};
use rocket_db_pools::sqlx;

pub mod handlers;
mod repo;
mod validators;

trait Validate {
    fn validate(&self) -> bool;
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
pub struct SignUp {
    username: String,
    password: String,
    password_check: String,
}

impl Validate for SignUp {
    fn validate(&self) -> bool {
        validators::is_valid_username(&self.username)
            && validators::is_valid_password(&self.password)
            && self.password == self.password_check
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SignIn {
    username: String,
    password: String,
}

impl Validate for SignIn {
    fn validate(&self) -> bool {
        validators::is_valid_username(&self.username)
            && validators::is_valid_password(&self.password)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    id: Uuid,
    username: String,
    password: String,
    pbkdf2_salt: String,
    created_at: Option<sqlx::types::chrono::NaiveDateTime>,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub pbkdf2_salt: String,
    pub created_at: Option<sqlx::types::chrono::NaiveDateTime>,
}

impl AuthenticatedUser {
    fn from_user(user: &User) -> Self {
        AuthenticatedUser {
            id: user.id,
            username: user.username.clone(),
            pbkdf2_salt: user.pbkdf2_salt.clone(),
            created_at: user.created_at,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header = req.headers().get_one("Authorization");
        let config = req.rocket().state::<Config>().unwrap();

        match auth_header {
            None => Outcome::Forward(Status::Unauthorized),
            Some(h) => {
                let parts = h.splitn(2, ' ').collect::<Vec<_>>();

                if parts.len() != 2 || parts[0].to_uppercase() != "BEARER" {
                    return Outcome::Forward(Status::Unauthorized);
                }

                let token = parts[1];
                let decode_result = jsonwebtoken::decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(config.access_token_secret.as_bytes()),
                    &Validation::new(jsonwebtoken::Algorithm::HS256),
                );

                match decode_result {
                    Ok(payload) => Outcome::Success(payload.claims.user),
                    Err(_) => Outcome::Forward(Status::Unauthorized),
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Claims {
    user: AuthenticatedUser,
    exp: usize,
}

impl Claims {
    pub fn encode(&self, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(&Header::default(), self, &EncodingKey::from_secret(secret))
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AccessToken {
    pub token: String,
}
