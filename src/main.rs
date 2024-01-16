mod data;
mod config;
mod auth;
mod session;

use auth::auth_routes;
use figment::providers::Format;
use rocket::fairing::AdHoc;
use rocket::{get, routes};
use rocket::fs::FileServer;
use figment::{Figment, Profile, providers::{Toml, Env}};
use rocket_db_pools::Database;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, TokenUrl, RedirectUrl, ClientId, ClientSecret};
use config::OauthConfig;
use data::GachaDb;
use session::storage::SessionStorage;
use session::SessionWriter;

#[get("/")]
fn index() -> String {
    format!("{:?}", "fightme")
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error>  {
    // Setup configuration sources
    let figment = Figment::from(rocket::Config::default())
        .merge(Toml::file("GachaConf.toml").nested())
        .merge(Env::prefixed("GACHA_").global())
        .select(Profile::from_env_or("GACHA_PROFILE", "default"));

    // Setup Oauth client
    let oauth_config = figment.extract::<OauthConfig>()
        .expect("Unable to launch due to missing Oauth Configuration.");
    let oauth_config = oauth_config.discord;
    
    let client_id = ClientId::new(oauth_config.client_id);
    let client_secret = ClientSecret::new(oauth_config.client_secret);
    let auth_url = AuthUrl::new(oauth_config.auth_url).expect("Invalid auth url");
    let token_url = TokenUrl::new(oauth_config.token_url).expect("Invalid token url");
    let redirect_url = RedirectUrl::new(oauth_config.redirect_url).expect("Invalid redirect url");

    let client = BasicClient::new(
        client_id,
        Some(client_secret),
        auth_url,
        Some(token_url)
    )
    .set_redirect_uri(redirect_url);

    let store_shutdown = AdHoc::on_shutdown("Shutdown Session", |rocket| Box::pin(async move {
        if let Some(store) = rocket.state::<SessionStorage>(){
            store.shutdown();
        }
    }));

    rocket::custom(figment)
        .mount("/api", routes![index])
        .mount("/auth", auth_routes())
        .mount("/", FileServer::from("./gacha_front/dist/"))
        .attach(GachaDb::init())
        .manage(client)
        .manage(SessionStorage::new())
        .attach(store_shutdown)
        .launch()
        .await?;

    Ok(())
}
