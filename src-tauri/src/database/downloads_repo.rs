use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: i64,
    pub file_name: String,
    pub url: String,
    pub save_path: String,
    pub status: String,
    pub created_at: i64,
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn record_download(
    conn: &Connection,
    file_name: &str,
    url: &str,
    save_path: &str,
    status: &str,
) -> Result<Vec<DownloadItem>, String> {
    let now = current_unix_timestamp();
    conn.execute(
        "INSERT INTO downloads (file_name, url, save_path, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![file_name, url, save_path, status, now],
    )
    .map_err(|e| format!("Failed to record download: {}", e))?;

    list_downloads(conn)
}

pub fn list_downloads(conn: &Connection) -> Result<Vec<DownloadItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, file_name, url, save_path, status, created_at
             FROM downloads
             ORDER BY created_at DESC, id DESC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(DownloadItem {
                id: row.get(0)?,
                file_name: row.get(1)?,
                url: row.get(2)?,
                save_path: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn clear_downloads(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM downloads", [])
        .map_err(|e| format!("Failed to clear downloads: {}", e))?;
    Ok(())
}
