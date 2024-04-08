use nanochat::{config::Config, db::Db};
use rocket::{
    fairing::AdHoc,
    figment::providers::{Format, Toml},
    launch,
};
use rocket_db_pools::Database;

#[launch]
fn rocket() -> _ {
    let figment = rocket::Config::figment().merge(Toml::file("App.toml").nested());

    rocket::custom(figment)
        .attach(AdHoc::config::<Config>())
        .attach(Db::init())
}
