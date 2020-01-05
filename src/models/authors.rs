use crate::schema::authors;
use crate::schema::authors::dsl::*;
use diesel::pg::upsert::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;

#[derive(Queryable)]
pub struct Author {
    pub id: i64,
    pub author: String,
}

#[derive(Insertable)]
#[table_name = "authors"]
pub struct NewAuthor<'a> {
    pub author: &'a str,
}

pub fn upsert_author(
    connection: &PgConnection,
    author_key: &str,
) -> Result<i32, diesel::result::Error> {
    diesel::insert_into(authors)
        .values(author.eq(author_key))
        .on_conflict(on_constraint("authors_author_unique_constraint"))
        .do_nothing()
        .returning(id)
        .execute(connection)?;

    authors
        .select(id)
        .filter(author.eq(author_key))
        .first(connection)
}