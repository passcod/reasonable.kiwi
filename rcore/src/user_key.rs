use crate::error::E;
use crate::plain::{self, Cipher, UserStack};
use crate::{schema, models};

use diesel::pg::PgConnection;
use diesel::prelude::*;
use futures::Stream;
use sodiumoxide::crypto::secretbox;
use std::env;
use tokio::runtime::current_thread::block_on_all;

const LISTKEYBYTES: usize = secretbox::KEYBYTES + secretbox::MACBYTES;

pub fn user_key(pan: &mut UserStack, egg: &egg_mode::user::TwitterUser, token: &egg_mode::Token, conn: &PgConnection, master_key: &secretbox::Key) -> Result<secretbox::Key, E> {
    let nonce = pan.plain.list_nonce;

    pan.plain.list().ok_or(E::NoListID)
        .and_then(|id| block_on_all(egg_mode::list::show(id, token)).map_err(E::Twitter))
        .or_else(|_| block_on_all(egg_mode::list::ownerships(&egg.id, token)
            .filter(|list| list.name.starts_with("Reasonable key")).collect()
        ).map_err(E::Twitter).and_then(|mut lists| lists.pop().ok_or(E::NoMatchingList)))
        .and_then(|list| match nonce {
            None => {
                if let Err(err) = block_on_all(egg_mode::list::delete(egg_mode::list::ListID::ID(list.id), token)) {
                    log::error!("Tried to delete list {} but failed: {}", list.id, err);
                }
                Err(E::NoMatchingList)
            },
            Some(_) => list.description.split_whitespace().last().map(|s| s.to_owned()).ok_or(E::NoDescription)
        })
        .and_then(|b65| base65536::decode(&b65, true).map_err(E::Encoding))
        .and_then(|key| if key.len() != LISTKEYBYTES {
            Err(E::KeyTooShort)
        } else {
            Ok(key)
        })
        .and_then(|enckey| secretbox::open(&enckey, &nonce.unwrap(), &master_key).map_err(|_| E::CannotDecrypt))
        .and_then(|key| secretbox::Key::from_slice(&key).ok_or(E::BadKeyFormat))
        .or_else(|_| {
            let key = secretbox::gen_key();
            let non = secretbox::gen_nonce();
            let enckey = secretbox::seal(&key[..], &non, &master_key);
            let b65: String = base65536::encode(&enckey, None);

            let list = block_on_all(egg_mode::list::create(
                "Reasonable key (pls keep)",
                false, // private
                Some(&format!("Don’t change! ➡️ https://reasonable.kiwi/help/list-key {}", b65)),
                &token
            ))?;

            pan.plain.list_nonce = Some(non);
            pan.plain.list_id = Some(list.id.into());
            let update = pan.cipher.update(&pan.plain, master_key).ok_or(E::CannotUpdateUser)?;

            {
                use schema::users::dsl::*;
                diesel::update(users.find(pan.cipher.id)).set(&update).execute(conn)?;
            }

            Ok(key)
        })
}

pub fn insert_env_user(master_key: &secretbox::Key, conn: &PgConnection) {
    let user = plain::User {
        access_keys: Some((env::var("TWITTER_USER_KEY").unwrap(), env::var("TWITTER_USER_SECRET").unwrap())),
        twitter_id: 92200252,
        list_id: None,
        list_nonce: None,
    };
    let user = models::User::new(&user, master_key);

    use schema::users::dsl::*;
    diesel::insert_into(users)
        .values(&user)
        .execute(conn)
        .unwrap();
}
