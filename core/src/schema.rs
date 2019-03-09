table! {
    reasons (id) {
        id -> Uuid,
        user_id -> Uuid,
        nonce -> Bytea,
        data -> Bytea,
    }
}

table! {
    users (id) {
        id -> Uuid,
        twitter_id -> Text,
        email -> Nullable<Text>,
        list_id -> Nullable<Text>,
        access_nonce -> Nullable<Bytea>,
        access_keys -> Nullable<Bytea>,
        list_nonce -> Nullable<Bytea>,
    }
}

joinable!(reasons -> users (user_id));

allow_tables_to_appear_in_same_query!(
    reasons,
    users,
);
