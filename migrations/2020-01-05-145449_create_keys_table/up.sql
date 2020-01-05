-- Your SQL goes here
CREATE TABLE IF NOT EXISTS keys (
  id SERIAL PRIMARY KEY,
  key TEXT UNIQUE NOT NULL
);
ALTER TABLE keys ADD CONSTRAINT keys_key_unique_constraint UNIQUE (key);
