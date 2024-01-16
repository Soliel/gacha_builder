use oauth2::{basic::BasicClient, PkceCodeChallenge, CsrfToken, Scope, PkceCodeVerifier, AuthorizationCode, reqwest::async_http_client, TokenResponse};
use reqwest::Client;
use rocket::{State, get, response::Redirect, Route, routes};

use crate::{SessionWriter, session::Session}; 

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

pub fn auth_routes() -> Vec<Route> {
    routes![discord, discord_redirect]
}