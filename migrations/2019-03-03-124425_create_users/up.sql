CREATE TABLE users (
    id uuid PRIMARY KEY,
    twitter bytea NOT NULL,
    data_version int NOT NULL DEFAULT 1,
    nonce bytea NOT NULL,
    data bytea NOT NULL
);
