use oauth2::{basic::BasicClient, PkceCodeChallenge, CsrfToken, Scope, PkceCodeVerifier, AuthorizationCode, reqwest::async_http_client, TokenResponse};
use reqwest::Client;
use rocket::{State, get, response::Redirect, Route, routes, uri, http::uri::{Uri, Origin, self}, serde::{json::Json, self}};
use ::serde::{Serialize, Deserialize};

use crate::{SessionWriter, session::Session}; 

struct ReturnTo(Origin<'static>);
pub struct DiscordToken(String);

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DiscordUser {
    id: String,
    username: String,
    email: String,
    avatar: String
}

#[get("/discord?<returnto>")]
async fn discord(oauth: &State<BasicClient>, sess_writer: SessionWriter, returnto: Option<&str>) -> Redirect {
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
    if let Some(uri_string) = returnto {
        if let Ok(origin_uri) = uri::Origin::parse_owned(uri_string.to_owned()) {
            sess_writer.insert_session_data(ReturnTo(origin_uri)).await;
        }
    }

    Redirect::to(auth_url.to_string())
}

#[get("/discord/redirect?<code>&<state>")]
async fn discord_redirect(
    client: &State<BasicClient>,
    csrf: Session<CsrfToken>, 
    pkce: Session<PkceCodeVerifier>,
    return_to: Option<Session<ReturnTo>>,
    sess_writer: SessionWriter,
    state: String, 
    code: String
) -> Result<Redirect, &'static str> {
    if !csrf.secret().eq(&state) {
        sess_writer.insert_session_data(CsrfToken::new("invalid".to_owned())).await;
        return Err("CSRF did not match")
    }

    let code = AuthorizationCode::new(code);
    let token = client
        .exchange_code(code)
        .set_pkce_verifier(PkceCodeVerifier::new(pkce.secret().clone())) //gross but it should work
        .request_async(async_http_client)
        .await;

    let token = match token {
        Ok(val) => val.access_token().secret().to_owned(),
        Err(_) => return Err("Unable to get access token")
    };

    // TODO: Insert auth info into DB
    sess_writer.insert_session_data(DiscordToken(token)).await;
    let redirect_origin = match return_to {
        Some(val) => val.0.to_owned(),
        None => uri!("/")
    };

    Ok(Redirect::to(redirect_origin))


}

#[get("/discord/me")]
pub async fn discord_me(token: Session<DiscordToken>) -> Result<Json<DiscordUser>, String> {
    let req_client = Client::new();
    let resp = req_client.get("https://discord.com/api/users/@me")
        .bearer_auth(&token.0)
        .send()
        .await;

    let resp = match resp {
        Ok(val) => val.json::<DiscordUser>().await,
        Err(e) => return Err(format!("Error: {}", e))
    };

    match resp {
        Ok(val) => Ok(Json(val)),
        Err(e) => return Err(format!("Error: {}", e))
    }
}

pub fn auth_routes() -> Vec<Route> {
    routes![discord, discord_redirect, discord_me]
}