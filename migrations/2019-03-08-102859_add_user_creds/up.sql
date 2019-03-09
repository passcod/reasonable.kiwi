ALTER TABLE users
    ADD COLUMN access_nonce bytea,
    ADD COLUMN access_keys bytea,
    ADD COLUMN list_nonce bytea;
