#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use sodiumoxide::crypto::{box_ as abox, sealedbox, secretbox};
use std::env;

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

fn main() {
    dotenv().expect("Dotenv failed to load");
    sodiumoxide::init().expect("Sodium failed to init");

    let master_key = master_key();
    let conn = database();

    some_users(&conn);
    some_reasons(&conn, &master_key);

    let reason = plain::Reason::new(plain::Action::Note, "good cunt");
    println!("{:?}", reason);
    let reason = models::Reason::default().update(&master_key, &reason).unwrap();
    println!("{:?}", reason);
    let reason = reason.open(&master_key);
    println!("{:?}", reason);
}
