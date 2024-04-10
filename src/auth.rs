use crate::{config::Config, db::Db, utils};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rocket::{
    http::{Cookie, CookieJar, Status},
    request::{FromRequest, Outcome},
    response::status::Created,
    serde::{json::Json, uuid::Uuid, Deserialize, Serialize},
    Request, State,
};
use rocket_db_pools::{sqlx, Connection};
use std::sync::Arc;

fn is_valid_username(username: &str) -> bool {
    username.len() >= 2
        && username.len() <= 32
        && username
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '-' || c == '_')
}

fn is_valid_password(password: &str) -> bool {
    let alphabetic_count = password.chars().filter(|c| c.is_alphabetic()).count();
    let ascii_digit_count = password.chars().filter(|c| c.is_ascii_digit()).count();
    let other_count = password.len() - alphabetic_count - ascii_digit_count;

    password.len() >= 12
        && password.len() <= 64
        && alphabetic_count > 0
        && ascii_digit_count > 0
        && other_count > 0
}

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
        is_valid_username(&self.username)
            && is_valid_password(&self.password)
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
        is_valid_username(&self.username) && is_valid_password(&self.password)
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

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Claims {
    user: AuthenticatedUser,
    exp: usize,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AccessToken {
    pub token: String,
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

#[rocket::post("/signup", data = "<body>")]
pub async fn signup(
    mut db: Connection<Db>,
    body: Json<SignUp>,
    config: &State<Config>,
) -> Result<Created<Json<AuthenticatedUser>>, Status> {
    if !body.validate() {
        return Err(Status::UnprocessableEntity);
    }

    let body = Arc::new(body);
    let body_clone = body.clone();
    let argon_secret_clone = config.argon_secret.clone();

    let password_hash = rocket::tokio::task::spawn_blocking(move || {
        let argon = Argon2::new_with_secret(
            argon_secret_clone.as_bytes(),
            Algorithm::Argon2id,
            Version::V0x13,
            Params::default(),
        )
        .map_err(|_| Status::InternalServerError)?;

        let salt = SaltString::generate(&mut OsRng);

        match argon.hash_password(body_clone.password.as_bytes(), &salt) {
            Err(_) => Err(Status::InternalServerError),
            Ok(h) => Ok(h.to_string()),
        }
    })
    .await
    .map_err(|_| Status::InternalServerError)??;

    let pbkdf2_salt = utils::compute_random_32_bytes_key();
    let user = sqlx::query_as!(
        AuthenticatedUser,
        r#"
        INSERT INTO users (username, password, pbkdf2_salt)
        VALUES ($1, $2, $3)
        RETURNING
            id,
            username,
            pbkdf2_salt,
            created_at;
        "#,
        body.clone().username,
        password_hash,
        pbkdf2_salt
    )
    .fetch_one(&mut **db)
    .await
    .map_err(|e| match e.as_database_error() {
        Some(e) if e.is_unique_violation() => Status::Conflict,
        _ => Status::InternalServerError,
    })?;

    Ok(Created::new("/signup").body(Json(user)))
}

#[rocket::post("/signin", data = "<body>")]
pub async fn signin(
    mut db: Connection<Db>,
    cookies: &CookieJar<'_>,
    body: Json<SignIn>,
    config: &State<Config>,
) -> Result<Json<AccessToken>, Status> {
    if !body.validate() {
        return Err(Status::UnprocessableEntity);
    }

    let body = Arc::new(body);
    let body_clone = body.clone();
    let argon_secret_clone = config.argon_secret.clone();

    let user = Arc::new(
        sqlx::query_as!(
            User,
            r"SELECT * FROM users WHERE username = $1;",
            body.clone().username,
        )
        .fetch_optional(&mut **db)
        .await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::Unauthorized)?,
    );

    let user_clone = user.clone();

    rocket::tokio::task::spawn_blocking(move || {
        let argon = Argon2::new_with_secret(
            argon_secret_clone.as_bytes(),
            Algorithm::Argon2id,
            Version::V0x13,
            Params::default(),
        )?;
        let password_hash = PasswordHash::new(&user_clone.password)?;
        argon.verify_password(body_clone.password.as_bytes(), &password_hash)
    })
    .await
    .map_err(|_| Status::InternalServerError)?
    .map_err(|_| Status::Unauthorized)?;

    if let Some(c) = cookies.get_private("session") {
        let result = sqlx::query!(
            r#"
            DELETE FROM sessions
            WHERE user_id = $1 AND NOT EXISTS(
                SELECT 1
                FROM sessions
                WHERE token = $2
            );
            "#,
            user.id,
            c.value()
        )
        .execute(&mut **db)
        .await;

        match result {
            Err(_) => return Err(Status::InternalServerError),
            Ok(r) if r.rows_affected() == 0 => {
                cookies.remove_private("session");
            }
            _ => {}
        }
    }

    let now = chrono::Utc::now().timestamp() as usize;
    let access_token = jsonwebtoken::encode(
        &Header::default(),
        &Claims {
            user: AuthenticatedUser::from_user(&user),
            exp: now + config.access_token_ttl_sec as usize,
        },
        &EncodingKey::from_secret(config.access_token_secret.as_bytes()),
    )
    .map_err(|_| Status::InternalServerError)?;

    let refresh_token = jsonwebtoken::encode(
        &Header::default(),
        &Claims {
            user: AuthenticatedUser::from_user(&user),
            exp: now + config.refresh_token_ttl_sec as usize,
        },
        &EncodingKey::from_secret(config.refresh_token_secret.as_bytes()),
    )
    .map_err(|_| Status::InternalServerError)?;

    sqlx::query!(
        r"INSERT INTO sessions (token, user_id) VALUES ($1, $2);",
        refresh_token,
        user.id,
    )
    .execute(&mut **db)
    .await
    .map_err(|e| {
        println!("{}", e);
        Status::InternalServerError
    })?;

    cookies.add_private(Cookie::build(("session", refresh_token)).max_age(
        rocket::time::Duration::seconds(config.refresh_token_ttl_sec as i64),
    ));

    Ok(Json(AccessToken {
        token: access_token,
    }))
}

#[rocket::post("/refresh")]
pub fn refresh(mut db: Connection<Db>, user: AuthenticatedUser, cookies: &CookieJar<'_>) {}

#[rocket::post("/logout")]
pub fn logout(mut db: Connection<Db>, user: AuthenticatedUser, cookies: &CookieJar<'_>) {}
