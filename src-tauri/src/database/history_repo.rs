use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub visited_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryCursor {
    pub visited_at: i64,
    pub id: i64,
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn record_visit(conn: &Connection, title: &str, url: &str) -> Result<(), String> {
    if url.trim().is_empty() || url.starts_with("about:") {
        return Ok(());
    }

    let now = current_unix_timestamp();
    conn.execute(
        "INSERT INTO history (title, url, visited_at) VALUES (?1, ?2, ?3)",
        params![title, url, now],
    )
    .map_err(|e| format!("Failed to record history visit: {}", e))?;

    Ok(())
}

pub fn get_recent_history(conn: &Connection, limit: usize) -> Result<Vec<HistoryEntry>, String> {
    search_history(conn, limit, "", None)
}

pub fn search_history(
    conn: &Connection,
    limit: usize,
    query: &str,
    before: Option<&HistoryCursor>,
) -> Result<Vec<HistoryEntry>, String> {
    let pattern = format!(
        "%{}%",
        query
            .trim()
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_")
    );
    let mut stmt = conn
        .prepare(
            "SELECT id, title, url, visited_at
             FROM history
             WHERE (?2 = '%%' OR title LIKE ?2 ESCAPE '!' OR url LIKE ?2 ESCAPE '!')
               AND (?3 IS NULL OR (visited_at, id) < (?3, ?4))
             ORDER BY visited_at DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(
            params![
                limit.clamp(1, 200) as i64,
                pattern,
                before.map(|cursor| cursor.visited_at),
                before.map(|cursor| cursor.id)
            ],
            |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    visited_at: row.get(3)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn searches_older_visits_and_paginates_equal_timestamps_without_duplicates() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE history (id INTEGER PRIMARY KEY, title TEXT, url TEXT, visited_at INTEGER); CREATE INDEX idx_history_recent ON history(visited_at DESC, id DESC);").unwrap();
        for id in 1..=300 {
            conn.execute(
                "INSERT INTO history VALUES (?1, ?2, 'https://example.test', 100)",
                params![
                    id,
                    if id == 1 {
                        "Older 100%_match"
                    } else {
                        "Recent"
                    }
                ],
            )
            .unwrap();
        }
        let first = search_history(&conn, 60, "", None).unwrap();
        let cursor = HistoryCursor {
            visited_at: first[59].visited_at,
            id: first[59].id,
        };
        let next = search_history(&conn, 60, "", Some(&cursor)).unwrap();
        assert_eq!(next[0].id, 240);
        assert_eq!(
            search_history(&conn, 60, "100%_match", None).unwrap()[0].id,
            1
        );
        assert_eq!(
            search_history(&conn, usize::MAX, "", None).unwrap().len(),
            200
        );
        let plan: String = conn.query_row("EXPLAIN QUERY PLAN SELECT * FROM history ORDER BY visited_at DESC, id DESC LIMIT 60", [], |row| row.get(3)).unwrap();
        assert!(plan.contains("idx_history_recent"));
        assert!(!plan.contains("TEMP B-TREE"));
    }
}

pub fn clear_all_history(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM history", [])
        .map_err(|e| format!("Failed to clear history: {}", e))?;
    Ok(())
}
