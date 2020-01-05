table! {
    authors (id) {
        id -> Int4,
        author -> Text,
    }
}

table! {
    keys (id) {
        id -> Int4,
        key -> Text,
    }
}

table! {
    messages (author_id, feed_id, seq) {
        seq -> Int4,
        key_id -> Int4,
        author_id -> Int4,
        feed_id -> Int4,
        entry -> Text,
        payload -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(authors, keys, messages,);
