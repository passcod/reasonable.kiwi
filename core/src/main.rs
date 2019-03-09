#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use sodiumoxide::crypto::{box_ as abox, sealedbox, secretbox};
use std::env;
use tokio::runtime::current_thread::block_on_all;
use futures::Stream;

pub mod models;
pub mod plain;
pub mod schema;

fn master_key() -> secretbox::Key {
    let arch_keys: (abox::PublicKey, abox::SecretKey) =
        Some(env!("ARCH_KEYS"))
            .and_then(|b64| base64::decode(&b64).ok())
            .and_then(|bin| bincode::deserialize(&bin).ok())
            .expect("ARCH_KEYS were not ready");

    env::var("MASTER_KEY").ok()
        .and_then(|b64| base64::decode(&b64).ok())
        .and_then(|enc: Vec<u8>| sealedbox::open(&enc, &arch_keys.0, &arch_keys.1).ok())
        .and_then(|dec| secretbox::Key::from_slice(&dec))
        .expect("MASTER_KEY must be set properly")
}

fn database() -> PgConnection {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&url).expect("Error connecting to database")
}

fn some_users(conn: &PgConnection) {
    use schema::users::dsl::*;
    let results = users
        .limit(5)
        .load::<models::User>(conn)
        .expect("Error loading users");

    println!("Displaying {} users", results.len());
    for user in results {
        println!("{:?}\n", user);
    }
}

fn some_reasons(conn: &PgConnection, key: &secretbox::Key) {
    use schema::reasons::dsl::*;
    let results = reasons
        .limit(5)
        .load::<models::Reason>(conn)
        .expect("Error loading reasons");

    println!("Displaying {} reasons", results.len());
    for reason in results {
        println!("Raw: {:?}", reason);
        println!("Dec: {:?}", reason.open(key));
    }
}

const LISTKEYBYTES: usize = secretbox::KEYBYTES + secretbox::MACBYTES;

#[derive(Debug)]
enum E {
    NoListID,
    NoMatchingList,
    NoDescription,
    KeyTooShort,
    BadKeyFormat,
    CannotDecrypt,
    Twitter(egg_mode::error::Error),
    Encoding(base65536::Error),
    Database(diesel::result::Error),
}

impl From<egg_mode::error::Error> for E {
    fn from(err: egg_mode::error::Error) -> Self {
        E::Twitter(err)
    }
}

impl From<base65536::Error> for E {
    fn from(err: base65536::Error) -> Self {
        E::Encoding(err)
    }
}

impl From<diesel::result::Error> for E {
    fn from(err: diesel::result::Error) -> Self {
        E::Database(err)
    }
}

fn user_key(pan: &models::User, egg: &egg_mode::user::TwitterUser, token: &egg_mode::Token, conn: &PgConnection, master_key: &secretbox::Key) -> Result<secretbox::Key, E> {
    let nonce = pan.get_list_nonce();

    pan.get_list_id().ok_or(E::NoListID)
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

            {
                use schema::users::dsl::*;
                diesel::update(users.find(pan.id)).set((
                    list_nonce.eq(non.as_ref().to_owned()),
                    list_id.eq(format!("{}", list.id))
                )).execute(conn)?;
            }

            Ok(key)
        })
}

fn main() {
    dotenv().expect("Dotenv failed to load");
    sodiumoxide::init().expect("Sodium failed to init");

    let master_key = master_key();
    let conn = database();

    // some_users(&conn);
    // some_reasons(&conn, &master_key);

    let app_token = egg_mode::KeyPair::new(env!("TWITTER_APP_KEY"), env!("TWITTER_APP_SECRET"));
    let pan_id = uuid::Uuid::parse_str("2ba595bf-cfaa-4e8b-8aeb-4268724aeedc").unwrap();

    /*
    let keys: (String, String) = (env::var("TWITTER_USER_KEY").unwrap(), env::var("TWITTER_USER_SECRET").unwrap());
    let keys = bincode::serialize(&keys).unwrap();
    let nonce = secretbox::gen_nonce();

    let keys = secretbox::seal(&keys, &nonce, &master_key);
    let nonce: Vec<u8> = (&nonce[..]).into();

    {
        use schema::users::dsl::*;
        diesel::update(users.find(pan_id)).set((
            access_nonce.eq(nonce),
            access_keys.eq(keys)
        )).execute(&conn);
    }
    return; */

    let pan = {
        use schema::users::dsl::*;
        users.filter(id.eq(pan_id)).first::<models::User>(&conn).unwrap()
    };

    let token = egg_mode::Token::Access {
        consumer: app_token,
        access: pan.decrypt_access(&master_key).unwrap(),
    };

    let egg = block_on_all(egg_mode::verify_tokens(&token)).expect("tokens are invalid");
    println!("User: “{}” @{} ({} / {})", egg.name, egg.screen_name, egg.id, pan.id);

    println!("{:?}", user_key(&pan, &egg, &token, &conn, &master_key));
}
