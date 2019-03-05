use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

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
    pub version: u8,
    pub action: Action,
    pub text: String, // max 500 chars
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Reason {
    pub fn new(action: Action, text: &str) -> Self {
        let now = Utc::now();
        Self {
            version: 1,
            action,
            text: text.into(),
            created: now,
            updated: now,
        }
    }
}
