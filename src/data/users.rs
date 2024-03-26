use diesel::prelude::*;
use uuid::Uuid;

use crate::{schema::users, auth::DiscordUserInfo};

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    email: &'a str
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct MinimalUser {
    pub id: Uuid,
    pub email: String
}


impl<'a> From<&'a DiscordUserInfo> for NewUser<'a> {
    fn from(value: &'a DiscordUserInfo) -> Self {
        Self { 
            email: &value.email
        }
    }
}
