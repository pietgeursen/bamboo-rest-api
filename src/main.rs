#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;
#[macro_use]
extern crate rocket_contrib;
extern crate dotenv;
#[macro_use]
extern crate diesel;

use bamboo_core::entry::decode;
use bamboo_core::YamfSignatory;
use bamboo_core::{lipmaa, verify};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use hex::{decode as decode_hex, encode as encode_hex};
use rocket::config::{Config, Environment};
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};

pub mod models;
pub mod schema;

use models::authors::*;
use models::messages::*;

pub fn establish_connection() -> Arc<Mutex<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    Arc::new(Mutex::new(connection))
}

#[derive(Deserialize, Serialize, JsonSchema)]
struct Entry {
    #[serde(rename = "encodedEntry")]
    encoded_entry: String,
    #[serde(rename = "encodedPayload")]
    encoded_payload: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct MyResult {
    res: String,
}

/// Gets a list of
#[openapi]
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
    let author = &decoded.author;

    let connection = state.lock().unwrap();
    let author_key = match author {
        YamfSignatory::Ed25519(pub_key, _) => upsert_author(&connection, &encode_hex(pub_key))
            .map_err(|e| json!({"errorUpsertingAuthorKey": e.to_string()}))?,
    };

    let previous_msg = get_message(
        &connection,
        author_key,
        decoded.seq_num as i64 - 1,
        decoded.log_id as i64,
    )
    .map_err(|e| json!({"errorGettingPreviousMessage": e.to_string()}))?
    .map(|msg| msg.entry)
    .map(|msg| decode_hex(msg).unwrap());

    let lipmaa_msg = get_message(
        &connection,
        author_key,
        lipmaa(decoded.seq_num) as i64,
        decoded.log_id as i64,
    )
    .map_err(|e| json!({"errorGettingLipmaaLink": e.to_string()}))?
    .map(|msg| msg.entry)
    .map(|msg| decode_hex(msg).unwrap());

    verify(
        &entry_bytes,
        Some(&payload_bytes),
        lipmaa_msg.as_deref(),
        previous_msg.as_deref(),
    )
    .map_err(|e| json!({ "errorVerifyingEntry": e }))?;

    let new_message = NewMessage {
        seq: decoded.seq_num as i64,
        author_id: author_key,
        feed_id: decoded.log_id as i64,
        entry: &entry.encoded_entry,
        payload: &entry.encoded_payload,
    };

    insert_message(&connection, &new_message)
        .map_err(|e| json!({"errorInsertingMessage": e.to_string()}))?;

    Ok(json!({
        "entry": entry.encoded_entry,
        "payload": entry.encoded_payload,
        "decoded": decoded
    }))
}

/// Gets a list of pub keys
#[openapi]
#[get("/")]
fn feeds(state: State<Arc<Mutex<PgConnection>>>) -> Result<JsonValue, JsonValue> {
    let connection = state.lock().unwrap();
    let authors =
        get_authors(&connection).map_err(|e| json!({"errorGettingAuthors": e.to_string()}))?;
    Ok(json!({ "authors": authors }))
}

/// Gets a list of feed ids by this pub key
#[openapi]
#[get("/<pub_key>")]
fn feeds_key(
    state: State<Arc<Mutex<PgConnection>>>,
    pub_key: String,
) -> Result<JsonValue, JsonValue> {
    let connection = state.lock().unwrap();
    let feed_ids = get_author_feed_ids(&connection, &pub_key)
        .map_err(|e| json!({"errorGettingAuthorFeeds": e.to_string()}))?;

    Ok(json!({ "feedIds": feed_ids }))
}

/// Gets all the messages by this pub_key published to this feed id
#[openapi]
#[get("/<pub_key>/<feed_id>")]
fn feeds_key_feed_id(
    state: State<Arc<Mutex<PgConnection>>>,
    pub_key: String,
    feed_id: i64,
) -> Result<JsonValue, JsonValue> {
    let connection = state.lock().unwrap();

    let author_id = get_author(&connection, &pub_key)
        .map_err(|e| json!({"errorGettingAuthorId": e.to_string()}))?;
    let messages = get_messages(&connection, author_id, feed_id)
        .map_err(|e| json!({"errorGettingAuthors": e.to_string()}))?
        .iter()
        .map(|msg| {
            let decoded_bytes = decode_hex(&msg.entry).unwrap();
            json!({
                "decoded": decode(&decoded_bytes).unwrap(),
                "encoded": &msg.entry,
                "payload": &msg.payload
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({ "messages": messages }))
}

/// Gets the message
#[openapi]
#[get("/<pub_key>/<feed_id>/<seq>")]
fn feeds_key_feed_id_seq(
    state: State<Arc<Mutex<PgConnection>>>,
    pub_key: String,
    feed_id: i64,
    seq: i64,
) -> Result<JsonValue, JsonValue> {
    let connection = state.lock().unwrap();

    let author_id = get_author(&connection, &pub_key)
        .map_err(|e| json!({"errorGettingAuthorId": e.to_string()}))?;

    let msg = get_message(&connection, author_id, seq, feed_id)
        .map_err(|e| json!({"errorGettingAuthorId": e.to_string()}))?
        .ok_or_else(|| json!({"errorNoAuthorFound": true}))?;

    let decoded_bytes = decode_hex(&msg.entry).unwrap();

    Ok(json!({
        "message": {
            "decoded": decode(&decoded_bytes).unwrap(),
            "encoded": &msg.entry,
            "payload": &msg.payload
        }
    }))
}

fn main() {
    let connection = establish_connection();

    let port = env::var("PORT")
        .unwrap_or_else(|_| 8000.to_string())
        .parse::<u16>()
        .unwrap();

    let config = Config::build(Environment::Development)
        .address("0.0.0.0")
        .port(port)
        .finalize()
        .unwrap();

    let swagger_config = rocket_okapi::swagger_ui::SwaggerUIConfig {
        url: Some("/openapi.json".to_owned()),
        urls: None,
    };

    let swagger_route = rocket_okapi::swagger_ui::make_swagger_ui(&swagger_config);

    rocket::custom(config)
        .manage(connection)
        .mount(
            "/",
            routes_with_openapi![
                feeds,
                feeds_key,
                feeds_key_feed_id,
                feeds_key_feed_id_seq,
                feeds_post
            ],
        )
        .mount("/swagger", swagger_route)
        .launch();
}
