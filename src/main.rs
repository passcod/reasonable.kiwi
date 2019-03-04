#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

pub mod schema;
pub mod models;
pub mod plain;

fn main() {
    dotenv().expect("Dotenv failed to load");
    sodiumoxide::init().expect("Sodium failed to init");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    {
        use schema::users::dsl::*;
        let results = users.filter(email.is_not_null())
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
