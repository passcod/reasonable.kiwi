use uuid::Uuid;
use sodiumoxide::crypto::secretbox::{Key, Nonce, open};
use crate::plain;

#[derive(Clone, Debug, Queryable)]
pub struct User {
    id: Uuid, // internal id
    twitter_id: String,
    email: Option<String>,
    list_id: Option<String>, // ? // key list cache
}

#[derive(Clone, Debug, Queryable)]
pub struct Reason {
    id: Uuid, // internal id
    user_id: Uuid,
    nonce: Vec<u8>, // encryption nonce
    data: Vec<u8>, // encrypted data
}

impl Reason {
    fn decrypt(&self, key: &Key) -> Option<plain::Reason> {
        let nonce = Nonce::from_slice(&self.nonce)?;
        let data = open(&self.data, &nonce, key).ok()?;
        let json = std::str::from_utf8(&data).ok()?;
        serde_json::from_str(json).ok()
    }
}
