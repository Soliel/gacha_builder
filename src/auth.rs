use diesel::upsert::on_constraint;
use oauth2::{basic::BasicClient, PkceCodeChallenge, CsrfToken, Scope, PkceCodeVerifier, AuthorizationCode, reqwest::async_http_client, TokenResponse};
use reqwest::Client;
use rocket::{State, get, response::Redirect, Route, routes, uri, http::uri::{Uri, Origin, self}, serde::{json::Json, self}};
use rocket_db_pools::Connection;
use rocket_db_pools::diesel::insert_into;
use rocket_db_pools::diesel::prelude::*;
use ::serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{SessionWriter, session::Session, data::{GachaDb, users::{NewUser, MinimalUser}}, schema::{self, discord_users}};

struct ReturnTo(Origin<'static>);
pub struct DiscordToken(String);


#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DiscordUserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub avatar: String
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DiscordOAuth {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64
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
    mut db: Connection<GachaDb>,
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

    let discord_info = get_discord_identity(&token).await;

    let user_id = if let Ok(discord_info) = discord_info {
        use crate::schema::users::dsl::*;

        let ins_result = insert_into(users)
            .values::<NewUser>((&discord_info).into())
            .returning(id)
            .get_result::<Uuid>(&mut db)
            .await;

        match ins_result {
            Ok(new_id) => Some(new_id),
            Err(_) => {
                let db_user = users
                    .filter(email.eq(&discord_info.email))
                    .select(MinimalUser::as_select())
                    .first(&mut db)
                    .await;

                match db_user {
                    Ok(val) => Some(val.id),
                    Err(_) => None
                }
            }
        }
    } else {
        None
    };

    let user_id = match user_id {
        Some(val) => val,
        None => return Err("Unable to get discord info")
    };


    sess_writer.insert_session_data(DiscordToken(token)).await;
    let redirect_origin = match return_to {
        Some(val) => val.0.to_owned(),
        None => uri!("/")
    };

    Ok(Redirect::to(redirect_origin))
}

#[get("/discord/me")]
pub async fn discord_me(token: Session<DiscordToken>) -> Result<Json<DiscordUserInfo>, String> {
    match get_discord_identity(&token.0).await {
        Ok(val) => Ok(Json(val)),
        Err(e) => Err(format!("Unable to get discord user info: {}", e))
    }
}

pub async fn get_discord_identity(token: &str) -> anyhow::Result<DiscordUserInfo>{
    let req_client = Client::new();
    let resp = req_client.get("https://discord.com/api/users/@me")
        .bearer_auth(token)
        .send()
        .await?;

    let resp = resp.json::<DiscordUserInfo>().await?;

    Ok(resp)
}

pub fn auth_routes() -> Vec<Route> {
    routes![discord, discord_redirect, discord_me]
}