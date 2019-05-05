sv[feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/auth/twitter")]
fn auth() {}

#[get("/")]
fn demo() -> &'static str {
    "Hi"
}

fn main() {
    println!("Hello, world!");
    rocket::ignite().mount("/", routes![hello, demo, auth]).launch();
}

