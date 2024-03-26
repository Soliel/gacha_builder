use rocket::serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct OauthConfig {
    pub discord: OauthClientConfig
}

#[derive(Debug,Deserialize, Serialize)]
pub struct OauthClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub revocation_url: String,
    pub token_secret: String
}