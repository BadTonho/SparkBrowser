use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkItem {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub icon: String,
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn seed_default_bookmarks_if_empty(conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))
        .unwrap_or(0);

    if count == 0 {
        let defaults = [
            ("DuckDuckGo", "https://duckduckgo.com", "🦆"),
            ("GitHub", "https://github.com", "🐙"),
            ("Rust Lang", "https://www.rust-lang.org", "🦀"),
            ("Wikipedia", "https://www.wikipedia.org", "🌐"),
        ];
        let now = current_unix_timestamp();
        for (title, url, icon) in defaults {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO bookmarks (title, url, icon, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![title, url, icon, now],
            );
        }
    }
    Ok(())
}

pub fn list_bookmarks(conn: &Connection) -> Result<Vec<BookmarkItem>, String> {
    let mut stmt = conn
        .prepare("SELECT id, title, url, icon FROM bookmarks ORDER BY id ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(BookmarkItem {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                icon: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for item in rows.flatten() {
        list.push(item);
    }
    Ok(list)
}

/// Toggles a bookmark by URL: removes it if it already exists, or inserts it if new.
pub fn toggle_bookmark(
    conn: &Connection,
    title: &str,
    url: &str,
    icon: &str,
) -> Result<Vec<BookmarkItem>, String> {
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM bookmarks WHERE url = ?1",
            params![url],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
    } else {
        let now = current_unix_timestamp();
        conn.execute(
            "INSERT INTO bookmarks (title, url, icon, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![title, url, icon, now],
        )
        .map_err(|e| e.to_string())?;
    }

    list_bookmarks(conn)
}

