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
    version: u8,
    action: Action,
    text: String, // max 500 chars
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}
