use chrono::{DateTime, Local};
use diesel::prelude::*;
use uuid::Uuid;
use crate::{schema::{discord_users, discord_oauth}, auth::DiscordUserInfo};


#[derive(Insertable, Identifiable)]
#[diesel(table_name = discord_users)]
pub struct NewDiscordUser<'a> {
    id: Uuid, 
    user_id: Uuid,
    discord_snowflake: &'a str,
    email: &'a str,
    avatar_hash: &'a str,
}

#[derive(Insertable, Identifiable)]
#[diesel(table_name = discord_oauth)]
#[primary_key(user_id)]
pub struct NewDiscordOauth<'a> {
    user_id: Uuid,
    access_token: &'a str,
    refresh_token: &'a str,
    expire_time: DateTime<Local>
}

impl<'a> NewDiscordUser<'a> {
    pub fn new(user_id: Uuid, user_info: &'a DiscordUserInfo) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            discord_snowflake: &user_info.id,
            email: &user_info.email,
            avatar_hash: &user_info.avatar
        }
    }
}