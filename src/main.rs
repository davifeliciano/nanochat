use nanochat::{
    auth::handlers::{logout, refresh, signin, signup},
    chat::handlers::insert_message,
    config::Config,
    db::Db,
    users::handlers::{accept, filtered_search, get_message_page, invite, search},
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
        .mount(
            "/users",
            routes![invite, accept, filtered_search, search, get_message_page],
        )
        .mount("/messages", routes![insert_message])
}
