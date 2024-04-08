use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("nanochat")]
pub struct Db(sqlx::PgPool);
