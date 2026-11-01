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
    let mut stmt = conn
        .prepare(
            "SELECT id, title, url, visited_at
             FROM history
             ORDER BY visited_at DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                visited_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn clear_all_history(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM history", [])
        .map_err(|e| format!("Failed to clear history: {}", e))?;
    Ok(())
}
