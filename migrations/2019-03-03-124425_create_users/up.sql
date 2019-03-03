CREATE TABLE users (
    id uuid PRIMARY KEY,
    twitter_id text NOT NULL,
    email text,
    list_id text
);
