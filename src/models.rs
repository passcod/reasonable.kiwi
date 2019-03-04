use uuid::Uuid;
use sodiumoxide::crypto::secretbox::{Key, Nonce, open, gen_nonce, seal};
use crate::plain;

#[derive(Clone, Debug, Queryable)]
pub struct User {
    id: Uuid, // internal id
    twitter_id: String,
    email: Option<String>,
    list_id: Option<String>, // ? // key list cache
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            twitter_id: "".into(),
            email: None,
            list_id: None,
        }
    }
}

#[derive(Clone, Debug, Queryable)]
pub struct Reason {
    id: Uuid, // internal id
    user_id: Uuid,
    nonce: Vec<u8>, // encryption nonce
    data: Vec<u8>, // encrypted data
}

impl Default for Reason {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: Uuid::default(),
            nonce: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl Reason {
    fn open(&self, key: &Key) -> Option<plain::Reason> {
        let nonce = Nonce::from_slice(&self.nonce)?;
        let bytes = open(&self.data, &nonce, key).ok()?;
        let json = std::str::from_utf8(&bytes).ok()?;
        serde_json::from_str(json).ok()
    }

    fn update(&self, key: &Key, data: &plain::Reason) -> Option<Self> {
        let nonce = gen_nonce();
        let bytes = serde_json::to_vec(data).ok()?;
        let data = seal(&bytes, &nonce, key);

        let mut reason = self.clone();
        reason.nonce = (&nonce[..]).into();
        reason.data = data;
        Some(reason)
    }
}
