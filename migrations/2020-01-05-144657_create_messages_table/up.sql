-- Your SQL goes here
CREATE TABLE IF NOT EXISTS messages (
  PRIMARY KEY(author_id, feed_id, seq),
  seq BIGINT NOT NULL,
  author_id INTEGER NOT NULL,
  feed_id BIGINT NOT NULL,
  entry TEXT NOT NULL,
  payload TEXT
);

CREATE INDEX IF NOT EXISTS messages_author_id_feed_id_index ON messages(author_id, feed_id);
CREATE INDEX IF NOT EXISTS messages_author_id_index ON messages(author_id);
CREATE INDEX IF NOT EXISTS messages_author_id_seq_index ON messages(author_id, seq);
