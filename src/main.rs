mod data;
mod config;
mod auth;
mod session;

use figment::providers::Format;
use rocket::response::Redirect;
use rocket::{get, routes, State, form};
use rocket::fs::FileServer;
use figment::{Figment, Profile, providers::{Toml, Env}};
use rocket_db_pools::Database;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, TokenUrl, RedirectUrl, ClientId, ClientSecret, 
    CsrfToken, TokenResponse, Scope, PkceCodeChallenge, PkceCodeVerifier, AuthorizationCode 
};
use oauth2::reqwest::async_http_client;
use reqwest::{Client};

use config::OauthConfig;
use data::GachaDb;
use session::{SessionStorage, SessionWriter, Session};

#[get("/")]
fn index() -> String {
    format!("{:?}", "fightme")
}

#[get("/discord")]
async fn discord(oauth: &State<BasicClient>, sess_writer: SessionWriter) -> Redirect {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token) = oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("identify".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();
    println!("{{\n\tauth: {}\n\tpkce: {:?}\n\tcsrf: {:?}\n}}", &auth_url, &pkce_verifier, &csrf_token);
    sess_writer.insert_session_data(pkce_verifier).await;
    sess_writer.insert_session_data(csrf_token).await;

    Redirect::to(auth_url.to_string())
}

#[get("/discord/redirect?<code>&<state>")]
async fn discord_redirect(
    client: &State<BasicClient>,
    csrf: Session<CsrfToken>, 
    pkce: Session<PkceCodeVerifier>, 
    state: String, 
    code: String
) -> String {
    println!("had: {} | received: {}", csrf.secret(), state);
    if !csrf.secret().eq(&state) {
        // Deny
    }

    let code = AuthorizationCode::new(code);
    let token = client
        .exchange_code(code)
        .set_pkce_verifier(PkceCodeVerifier::new(pkce.secret().clone())) //gross but it should work
        .request_async(async_http_client)
        .await;

    let token = match token {
        Ok(val) => val.access_token().secret().to_owned(),
        Err(e) => return format!("Error: {}", e)
    };

    let req_client = Client::new();
    let resp = req_client.get("https://discord.com/api/users/@me")
        .bearer_auth(token)
        .send()
        .await;
    let resp = match resp {
        Ok(val) => val.text().await,
        Err(e) => return format!("Error: {}", e)
    };

    match resp {
        Ok(val) => val,
        Err(e) => format!("Error: {}", e)
    }
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

    rocket::custom(figment)
        .mount("/api", routes![index])
        .mount("/auth", routes![discord, discord_redirect])
        .mount("/", FileServer::from("./gacha_front/dist/"))
        .attach(GachaDb::init())
        .attach(SessionStorage::new())
        .manage(client)
        .launch()
        .await?;

    Ok(())
}
