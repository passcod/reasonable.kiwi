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
    }
}

joinable!(reasons -> users (user_id));

allow_tables_to_appear_in_same_query!(reasons, users,);
