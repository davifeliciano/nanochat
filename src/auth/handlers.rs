use super::{repo, AccessToken, AuthenticatedUser, Claims, Validate};
use crate::{
    auth::{SignIn, SignUp},
    config::Config,
    db::Db,
    utils,
};
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use rand::rngs::OsRng;
use rocket::{
    http::{Cookie, CookieJar, Status},
    response::status::Created,
    serde::json::Json,
    State,
};
use rocket_db_pools::Connection;
use std::sync::Arc;

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
        .or(Err(Status::InternalServerError))?;

        let salt = SaltString::generate(&mut OsRng);

        match argon.hash_password(body_clone.password.as_bytes(), &salt) {
            Err(_) => Err(Status::InternalServerError),
            Ok(h) => Ok(h.to_string()),
        }
    })
    .await
    .or(Err(Status::InternalServerError))??;

    let pbkdf2_salt = utils::compute_random_32_bytes_key();
    let user = repo::insert_user(
        &mut db,
        &body.clone().username,
        &password_hash,
        &pbkdf2_salt,
    )
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
        repo::get_user_by_username(&mut db, &body_clone.username)
            .await
            .or(Err(Status::InternalServerError))?
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
    .or(Err(Status::InternalServerError))?
    .or(Err(Status::Unauthorized))?;

    if let Some(c) = cookies.get_private("session") {
        repo::remove_all_user_sessions_on_reuse(&mut db, user.id, c.value())
            .await
            .or(Err(Status::InternalServerError))?;
    }

    let now = chrono::Utc::now().timestamp() as usize;
    let mut claims = Claims {
        user: AuthenticatedUser::from_user(&user),
        exp: now + config.access_token_ttl_sec as usize,
    };

    let access_token = claims
        .encode(config.access_token_secret.as_bytes())
        .or(Err(Status::InternalServerError))?;

    claims.exp = now + config.refresh_token_ttl_sec as usize;

    let refresh_token = claims
        .encode(config.refresh_token_secret.as_bytes())
        .or(Err(Status::InternalServerError))?;

    repo::create_session(&mut db, user.id, &refresh_token)
        .await
        .or(Err(Status::InternalServerError))?;

    cookies.add_private(Cookie::build(("session", refresh_token)).max_age(
        rocket::time::Duration::seconds(config.refresh_token_ttl_sec as i64),
    ));

    Ok(Json(AccessToken {
        token: access_token,
    }))
}

#[rocket::post("/refresh")]
pub async fn refresh(
    mut db: Connection<Db>,
    user: AuthenticatedUser,
    cookies: &CookieJar<'_>,
    config: &State<Config>,
) -> Result<Json<AccessToken>, Status> {
    let session = cookies.get_private("session");
    let user_id = user.id;

    if let Some(ref c) = session {
        let result = repo::remove_all_user_sessions_on_reuse(&mut db, user.id, c.value()).await;

        match result {
            Err(_) => return Err(Status::InternalServerError),
            Ok(r) if r.rows_affected() != 0 => {
                cookies.remove_private("session");
                return Err(Status::Unauthorized);
            }
            _ => {}
        }
    } else {
        return Err(Status::Unauthorized);
    }

    let now = chrono::Utc::now().timestamp() as usize;
    let mut claims = Claims {
        user,
        exp: now + config.access_token_ttl_sec as usize,
    };

    let access_token = claims
        .encode(config.access_token_secret.as_bytes())
        .or(Err(Status::InternalServerError))?;

    claims.exp = now + config.refresh_token_ttl_sec as usize;

    let refresh_token = claims
        .encode(config.refresh_token_secret.as_bytes())
        .or(Err(Status::InternalServerError))?;

    repo::update_session(&mut db, user_id, &session.unwrap().value(), &refresh_token)
        .await
        .or(Err(Status::InternalServerError))?;

    cookies.add_private(Cookie::build(("session", refresh_token)).max_age(
        rocket::time::Duration::seconds(config.refresh_token_ttl_sec as i64),
    ));

    Ok(Json(AccessToken {
        token: access_token,
    }))
}

#[rocket::post("/logout")]
pub fn logout(mut db: Connection<Db>, user: AuthenticatedUser, cookies: &CookieJar<'_>) {}
