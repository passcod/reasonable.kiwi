CREATE TABLE reasons (
    id uuid PRIMARY KEY,
    user_id uuid references users(id) NOT NULL,
    nonce bytea NOT NULL,
    data bytea NOT NULL
);
