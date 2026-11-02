//! Fold once on writes; trigram candidates preserve literal substring matching.
use rusqlite::Connection;

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "command-search-trigrams-v1", |conn| {
        conn.execute_batch(include_str!("search_index.sql"))
            .map_err(|e| e.to_string())
    })
}

pub fn pattern(query: &str) -> Option<String> {
    (query.chars().count() >= 3).then(|| format!("\"{}\"", query.replace('"', "\"\"")))
}

#[cfg(test)]
#[path = "search_index_tests.rs"]
mod tests;
