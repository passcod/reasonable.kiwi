use diesel::prelude::*;
use tokio::runtime::current_thread::block_on_all;

fn main() {
    let (master_key, conn) = rcore::init();

    // some_users(&conn);
    // some_reasons(&conn, &master_key);

    let app_token = egg_mode::KeyPair::new(env!("TWITTER_APP_KEY"), env!("TWITTER_APP_SECRET"));
    let pan_id = uuid::Uuid::parse_str("23da46f5-7a61-456e-b0e7-3a6ea1abbd0e").unwrap();

    if false {
        rcore::user_key::insert_env_user(&master_key, &conn);
    }

    let pan = {
        use rcore::schema::users::dsl::*;
        users.filter(id.eq(pan_id)).first::<rcore::models::User>(&conn).unwrap()
    };
    let mut pan = pan.open(&master_key).unwrap();

    let token = egg_mode::Token::Access {
        consumer: app_token,
        access: pan.plain.keypair().unwrap(),
    };

    let egg = block_on_all(egg_mode::verify_tokens(&token)).expect("tokens are invalid");
    println!("User: “{}” @{} ({} / {})", egg.name, egg.screen_name, egg.id, pan.cipher.id);

    println!("{:?}", rcore::user_key::user_key(&mut pan, &egg, &token, &conn, &master_key));
}
