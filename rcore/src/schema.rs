table! {
    reasons (id) {
        id -> Uuid,
        user_id -> Uuid,
        data_version -> Int4,
        nonce -> Bytea,
        data -> Bytea,
    }
}

table! {
    users (id) {
        id -> Uuid,
        twitter -> Bytea,
        data_version -> Int4,
        nonce -> Bytea,
        data -> Bytea,
    }
}

joinable!(reasons -> users (user_id));

allow_tables_to_appear_in_same_query!(
    reasons,
    users,
);
