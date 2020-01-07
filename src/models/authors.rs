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

pub fn get_authors(connection: &PgConnection) -> Result<Vec<String>, diesel::result::Error> {
    authors.select(author).load(connection)
}

pub fn get_author(
    connection: &PgConnection,
    author_str: &str,
) -> Result<i32, diesel::result::Error> {
    authors
        .filter(author.eq(author_str))
        .select(id)
        .first(connection)
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

    get_author(connection, author_key)
}
