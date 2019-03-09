CREATE TABLE reasons (
    id uuid PRIMARY KEY,
    user_id uuid references users(id) NOT NULL,
    data_version int NOT NULL DEFAULT 1,
    nonce bytea NOT NULL,
    data bytea NOT NULL
);
