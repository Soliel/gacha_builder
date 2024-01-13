mod data;
mod config;
mod auth;
mod session;

use figment::providers::{Serialized, Format};
use rocket::fairing::AdHoc;
use rocket::local::asynchronous::Client;
use rocket::{get, launch, routes, State};
use rocket::fs::FileServer;
use figment::{Figment, Profile, providers::{Toml, Env}};
use rocket_db_pools::Database;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, TokenUrl, RedirectUrl, ClientId, ClientSecret, 
    CsrfToken, TokenResponse, Scope 
};

use config::OauthConfig;
use data::GachaDb;
use session::SessionStorage;

#[get("/")]
fn index() -> String {
    format!("{:?}", "fightme")
}

#[launch]
fn rocket() -> _ {
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

    rocket::custom(figment)
        .mount("/api", routes![index])
        .mount("/", FileServer::from("./gacha_front/dist/"))
        .attach(GachaDb::init())
        .attach(SessionStorage::new())
}
