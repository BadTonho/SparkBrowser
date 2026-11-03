use crate::database::history_repo::{search_history, HistoryCursor};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RecentSearch {
    pub query: String,
    pub engine: &'static str,
}

pub fn from_url(raw: &str) -> Option<RecentSearch> {
    let url = url::Url::parse(raw).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    let engine = match (url.host_str()?, url.path()) {
        ("google.com" | "www.google.com" | "google.com.br" | "www.google.com.br", "/search") => {
            "google"
        }
        (
            "duckduckgo.com" | "www.duckduckgo.com",
            "/" | "/html/" | "/html" | "/lite/" | "/lite",
        )
        | ("html.duckduckgo.com", "/html/" | "/html")
        | ("lite.duckduckgo.com", "/lite/" | "/lite") => "duckduckgo",
        _ => return None,
    };
    let query = url
        .query_pairs()
        .find(|(key, _)| key == "q")?
        .1
        .trim()
        .to_string();
    (!query.is_empty()).then_some(RecentSearch { query, engine })
}

pub fn list(conn: &Connection) -> Result<Vec<RecentSearch>, String> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut cursor = None;
    loop {
        let rows = search_history(conn, 200, "", cursor.as_ref())?;
        for row in &rows {
            if let Some(search) = from_url(&row.url) {
                if seen.insert(search.query.to_lowercase()) {
                    found.push(search);
                    if found.len() == 5 {
                        return Ok(found);
                    }
                }
            }
        }
        if rows.len() < 200 {
            return Ok(found);
        }
        cursor = rows.last().map(|row| HistoryCursor {
            visited_at: row.visited_at,
            id: row.id,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_search_hosts_paths_and_decodes_terms() {
        assert_eq!(
            from_url("https://www.google.com.br/search?q=Rust+%26+Svelte")
                .unwrap()
                .query,
            "Rust & Svelte"
        );
        for url in [
            "https://google.com.evil/search?q=x",
            "https://evil/google.com/search?q=x",
            "https://google.com/?q=x",
            "https://google.com/search?q=+",
            "https://user@google.com/search?q=x",
            "file:///search?q=x",
        ] {
            assert!(from_url(url).is_none(), "{url}");
        }
        for url in [
            "https://duckduckgo.com/?q=x",
            "https://html.duckduckgo.com/html/?q=x",
            "https://lite.duckduckgo.com/lite/?q=x",
        ] {
            assert_eq!(from_url(url).unwrap().engine, "duckduckgo");
        }
    }
    #[test]
    fn scans_beyond_one_page_and_deduplicates_newest_terms() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE history(id INTEGER PRIMARY KEY,title TEXT,url TEXT,visited_at INTEGER)",
        )
        .unwrap();
        for id in 1..=410 {
            db.execute(
                "INSERT INTO history VALUES(?1,'','https://example.test',?1)",
                [id],
            )
            .unwrap();
        }
        for (id, url) in [
            (1, "https://google.com/search?q=old"),
            (2, "https://google.com/search?q=Rust"),
            (3, "https://duckduckgo.com/?q=+RUST+"),
            (4, "https://google.com/search?q=third"),
            (5, "https://google.com/search?q=fourth"),
            (6, "https://google.com/search?q=fifth"),
        ] {
            db.execute(
                "UPDATE history SET url=?1 WHERE id=?2",
                rusqlite::params![url, id],
            )
            .unwrap();
        }
        let rows = list(&db).unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(
            rows[3],
            RecentSearch {
                query: "RUST".into(),
                engine: "duckduckgo"
            }
        );
        assert_eq!(rows[4].query, "old");
        db.execute("DELETE FROM history", []).unwrap();
        assert!(list(&db).unwrap().is_empty());
    }
}
