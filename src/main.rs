#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate dotenv;
#[macro_use]
extern crate diesel;

use serde::Deserialize;
use rocket_contrib::json::{JsonValue, Json};
use rocket::config::{Config, Environment};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use hex::{decode as decode_hex};
use std::env;
use bamboo_core::{verify, Entry as BambooEntry, lipmaa};
use bamboo_core::entry::decode;

pub mod models;
pub mod schema;

use models::authors::*;
use models::keys::*;
use models::messages::*;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

/// Gets a list of public keys we have feeds of.
#[get("/")]
fn feeds() -> JsonValue {
    json!({
        "feeds": "a feed"
    })
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "encodedEntry")]
    encoded_entry: String,
    #[serde(rename = "encodedPayload")]
    encoded_payload: String
}

/// Gets a list of
#[post("/", data = "<entry>")]
fn feeds_post(entry: Json<Entry>) -> JsonValue {

    let entry_bytes = decode_hex(&entry.encoded_entry).unwrap();
    let decoded_result = decode(&entry_bytes);
    json!({
        "entry": entry.encoded_entry,
        "payload": entry.encoded_payload,
        "didDecode": decoded_result
    })
}


/// Gets a list of
#[get("/<pub_key>")]
fn feeds_key(pub_key: String) -> JsonValue {
    json!({
        "feeds": "a feed"
    })
}

#[get("/<pub_key>/<feed_id>")]
fn feeds_key_feed_id(pub_key: String, feed_id: String) -> JsonValue {
    json!({
        "feeds": "a feed"
    })
}

#[get("/<pub_key>/<feed_id>/<seq>")]
fn feeds_key_feed_id_seq(pub_key: String, feed_id: String, seq: u64) -> JsonValue {
    json!({
        "feeds": "a feed"
    })
}

#[get("/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

fn main() {
    let connection = establish_connection();

    let res = upsert_author(&connection, "pietKey").unwrap();

    println!("got res {}", res);

    let res2 = upsert_key(&connection, "key").unwrap();

    println!("got res {}", res2);

    let new_message = NewMessage {
        seq: 1,
        key_id: res2,
        author_id: res,
        feed_id: 1,
        entry: "I'm the entry.",
        payload: "I'm the payload",
    };

    let res3 = insert_message(&connection, &new_message).unwrap();
    println!("got res {}", res3);

    let port = env::var("PORT").unwrap_or(8000.to_string()).parse::<u16>().unwrap();

    let config = Config::build(Environment::Production)
        .address("0.0.0.0")
        .port(port)
        .finalize().unwrap();

    rocket::custom(config)
        .mount("/hello", routes![hello])
        .mount("/feeds", routes![feeds, feeds_key, feeds_key_feed_id_seq, feeds_post])
        .launch();
}
