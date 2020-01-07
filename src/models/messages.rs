use crate::schema::messages;
use crate::schema::messages::dsl::*;
use diesel::pg::upsert::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;

#[derive(Queryable)]
pub struct Message {
    pub seq: i32,
    pub author_id: i32,
    pub feed_id: i32,
    pub entry: String,
    pub payload: Option<String>,
}

#[derive(Insertable)]
#[table_name = "messages"]
pub struct NewMessage<'a> {
    pub seq: i32,
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
pub fn get_messages(
    connection: &PgConnection,
    author: i32,
    feed: i32,
) -> Result<Vec<Message>, diesel::result::Error> {
    messages
        .filter(author_id.eq(author).and(feed_id.eq(feed)))
        .load::<Message>(connection)
}
pub fn get_message(
    connection: &PgConnection,
    author: i32,
    sequence: i32,
    feed: i32,
) -> Result<Option<Message>, diesel::result::Error> {
    messages
        .filter(
            seq.eq(sequence)
                .and(author_id.eq(author).and(feed_id.eq(feed))),
        )
        .first::<Message>(connection)
        .optional()
}
