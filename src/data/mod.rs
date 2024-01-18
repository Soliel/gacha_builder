pub mod discord;
pub mod users;

use rocket_db_pools::{Database, diesel};

#[derive(Database)]
#[database("gacha_db")]
pub struct GachaDb(diesel::PgPool);
