use crate::schema::messages;
use crate::schema::messages::dsl::*;
use diesel::pg::upsert::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;

#[derive(Queryable)]
pub struct Message {
    pub id: i64,
    pub seq: i32,
    pub key_id: i32,
    pub author_id: i32,
    pub feed_id: i32,
    pub entry: String,
    pub payload: String,
}

#[derive(Insertable)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub seq: i32,
    pub key_id: i32,
    pub author_id: i32,
    pub feed_id: i32,
    pub entry: &'a str,
    pub payload: &'a str,
}

pub fn insert_message(
    connection: &PgConnection,
    new_message: &NewMessage,
) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(messages)
        .values(new_message)
        .on_conflict(on_constraint("messages_pkey"))
        .do_nothing()
        .execute(connection)
}
