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

fn main() {
    dotenv().expect("Dotenv failed to load");
    sodiumoxide::init().expect("Sodium failed to init");

    let master_key: secretbox::Key = {
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
    };

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    {
        use schema::users::dsl::*;
        let results = users
            .filter(email.is_not_null())
            .limit(5)
            .load::<models::User>(&conn)
            .expect("Error loading users");

        println!("Displaying {} users", results.len());
        for user in results {
            println!("{:?}\n", user);
        }

        println!("{:?}", models::User::default());
    }
}
