use crate::schema::keys;
use crate::schema::keys::dsl::*;
use diesel::pg::upsert::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;

#[derive(Queryable)]
pub struct Key {
    pub id: i64,
    pub key: String,
}

#[derive(Insertable)]
#[table_name = "keys"]
pub struct NewKey<'a> {
    pub key: &'a str,
}

pub fn upsert_key(connection: &PgConnection, keys_key: &str) -> Result<i32, diesel::result::Error> {
    diesel::insert_into(keys)
        .values(key.eq(keys_key))
        .on_conflict(on_constraint("keys_key_unique_constraint"))
        .do_nothing()
        .returning(id)
        .execute(connection)?;

    keys.select(id).filter(key.eq(keys_key)).first(connection)
}
