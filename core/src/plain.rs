use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;

pub struct Stack<'cipher, C: Cipher, P: Serialize> {
    pub cipher: &'cipher C,
    pub plain: P,
}

pub trait Cipher {
    type Plain: Serialize;
    fn update(&self, plain: &Self::Plain, key: &secretbox::Key) -> Option<Self>
        where Self: Sized;
}

pub type ReasonStack<'c> = Stack<'c, crate::models::Reason, Reason>;
pub type UserStack<'c> = Stack<'c, crate::models::User, User>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Follow,
    Unfollow,
    Block,
    Mute,
    Note, // just anything
    Warning, // like a note but explicitely negative
    // TODO: List(String), // string = listname
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reason {
    pub action: Action,
    pub text: String, // max 500 chars
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Reason {
    pub fn new(action: Action, text: &str) -> Self {
        let now = Utc::now();
        Self {
            action,
            text: text.into(),
            created: now,
            updated: now,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub twitter_id: u64,
    pub list_id: Option<u64>,
    pub list_nonce: Option<secretbox::Nonce>,
    pub access_keys: Option<(String, String)>,
}

impl User {
    pub fn twitter(&self) -> egg_mode::user::UserID {
        egg_mode::user::UserID::ID(self.twitter_id)
    }

    pub fn list(&self) -> Option<egg_mode::list::ListID> {
        self.list_id.as_ref().map(|id| egg_mode::list::ListID::ID(*id))
    }

    pub fn keypair(&self) -> Option<egg_mode::KeyPair> {
        self.access_keys.as_ref().map(|(p, s)| egg_mode::KeyPair::new(p.to_string(), s.to_string()))
    }
}
