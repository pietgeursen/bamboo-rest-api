-- Your SQL goes here
CREATE TABLE IF NOT EXISTS authors (
  id SERIAL PRIMARY KEY,
  author TEXT NOT NULL
);
ALTER TABLE authors ADD CONSTRAINT authors_author_unique_constraint UNIQUE (author);
