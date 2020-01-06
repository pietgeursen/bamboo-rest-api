#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate dotenv;
#[macro_use]
extern crate diesel;

use bamboo_core::entry::decode;
use bamboo_core::YamfSignatory;
use bamboo_core::{lipmaa, verify, Entry as BambooEntry};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use hex::{decode as decode_hex, encode as encode_hex};
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use serde::Deserialize;
use std::env;
use std::sync::{Arc, Mutex};

pub mod models;
pub mod schema;

use models::authors::*;
use models::keys::*;
use models::messages::*;

pub fn establish_connection() -> Arc<Mutex<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    Arc::new(Mutex::new(connection))
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
    encoded_payload: String,
}

/// Gets a list of
#[post("/", data = "<entry>")]
fn feeds_post(
    entry: Json<Entry>,
    state: State<Arc<Mutex<PgConnection>>>,
) -> Result<JsonValue, JsonValue> {

    let entry_bytes = decode_hex(&entry.encoded_entry)
        .map_err(|e| json!({"errorDecodingEntryFromHexString": e.to_string()}))?;
    let payload_bytes = decode_hex(&entry.encoded_payload)
        .map_err(|e| json!({"errorDecodingPayloadFromHexString": e.to_string()}))?;

    let decoded = decode(&entry_bytes).map_err(|e| json!({ "errorDecodingEntry": e }))?;

    // TODO get the lipmaa and backlink entries

    verify(&entry_bytes, Some(&payload_bytes), None, None)
        .map_err(|e|{
            json!({"errorVerifyingEntry": e})
        })?; 

    let author = &decoded.author;
    let connection = state.lock().unwrap();

    let author_key = match author {
        YamfSignatory::Ed25519(pub_key, _) => {
            upsert_author(&connection, &encode_hex(pub_key))
                .map_err(|e| json!({"errorUpsertingAuthorKey": e.to_string()}))?
        }
    };

    let new_message = NewMessage {
        seq: decoded.seq_num as i32,
        key_id: 1, // TODO
        author_id: author_key,
        feed_id: decoded.log_id as i32,
        entry: &entry.encoded_entry,
        payload: &entry.encoded_payload,
    };

    insert_message(&connection, &new_message)
        .map_err(|e|{
            json!({"errorInsertingMessage": e.to_string()})
        })?;

    Ok(json!({
        "entry": entry.encoded_entry,
        "payload": entry.encoded_payload,
        "decoded": decoded
    }))
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

    //let res = upsert_author(&connection., "pietKey").unwrap();

    //println!("got res {}", res);

    //let res2 = upsert_key(&connection, "key").unwrap();

    //println!("got res {}", res2);

    //    let new_message = NewMessage {
    //        seq: 1,
    //        key_id: res2,
    //        author_id: res,
    //        feed_id: 1,
    //        entry: "I'm the entry.",
    //        payload: "I'm the payload",
    //    };
    //
    //    let res3 = insert_message(&connection, &new_message).unwrap();
    //    println!("got res {}", res3);

    let port = env::var("PORT")
        .unwrap_or(8000.to_string())
        .parse::<u16>()
        .unwrap();

    let config = Config::build(Environment::Production)
        .address("0.0.0.0")
        .port(port)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .manage(connection)
        .mount("/hello", routes![hello])
        .mount(
            "/feeds",
            routes![feeds, feeds_key, feeds_key_feed_id_seq, feeds_post],
        )
        .launch();
}
