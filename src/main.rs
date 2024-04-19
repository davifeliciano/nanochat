use nanochat::{
    auth::handlers::{logout, refresh, signin, signup},
    config::Config,
    db::Db,
    users::handlers::{accept, invite},
};
use rocket::{
    fairing::AdHoc,
    figment::providers::{Format, Toml},
    launch, routes,
};
use rocket_db_pools::Database;

#[launch]
fn rocket() -> _ {
    let figment = rocket::Config::figment().merge(Toml::file("App.toml").nested());

    rocket::custom(figment)
        .attach(AdHoc::config::<Config>())
        .attach(Db::init())
        .mount("/auth", routes![signup, signin, refresh, logout])
        .mount("/users", routes![invite, accept])
}
