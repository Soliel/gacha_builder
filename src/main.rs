mod data;
mod config;

use figment::providers::{Serialized, Format};
use rocket::fairing::AdHoc;
use rocket::{get, launch, routes, State};
use rocket::fs::FileServer;
use figment::{Figment, Profile, providers::{Toml, Env}};
use rocket_db_pools::Database;

use config::GachaConfig;
use data::GachaDb;

#[get("/")]
fn index(app_config: &State<GachaConfig>) -> String {
    format!("{:?}", app_config)
}

#[launch]
fn rocket() -> _ {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(GachaConfig::default()))
        .merge(Toml::file("GachaConf.toml").nested())
        .merge(Env::prefixed("GACHA_").global())
        .select(Profile::from_env_or("GACHA_PROFILE", "default"));
    
    rocket::custom(figment)
        .mount("/api", routes![index])
        .mount("/", FileServer::from("./gacha_front/dist/"))
        .attach(AdHoc::config::<GachaConfig>())
        .attach(GachaDb::init())
}
