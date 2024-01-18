// @generated automatically by Diesel CLI.

diesel::table! {
    discord_oauth (user_id) {
        user_id -> Uuid,
        #[max_length = 255]
        access_token -> Varchar,
        #[max_length = 255]
        refresh_token -> Varchar,
        expire_time -> Timestamptz,
        created_at -> Timestamptz,
        last_updated -> Timestamptz,
    }
}

diesel::table! {
    discord_users (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 64]
        discord_snowflake -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        avatar_hash -> Nullable<Varchar>,
        created_at -> Timestamptz,
        last_updated -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        created_at -> Timestamptz,
        last_updated -> Timestamptz,
    }
}

diesel::joinable!(discord_oauth -> users (user_id));
diesel::joinable!(discord_users -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    discord_oauth,
    discord_users,
    users,
);
