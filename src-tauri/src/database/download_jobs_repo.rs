use crate::downloads::model::DownloadRecord;
use rusqlite::{params, Connection};

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "resumable-downloads-v1", |conn| {
        conn.execute_batch("CREATE TABLE download_jobs(id TEXT PRIMARY KEY,document TEXT NOT NULL); ALTER TABLE downloads ADD COLUMN task_id TEXT; CREATE UNIQUE INDEX idx_download_task ON downloads(task_id) WHERE task_id IS NOT NULL;").map_err(|e|e.to_string())
    })
}
pub fn save(conn: &Connection, record: &DownloadRecord) -> Result<(), String> {
    let json = serde_json::to_string(record).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO download_jobs VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET document=excluded.document",params![record.task.id,json]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn load(conn: &Connection) -> Result<Vec<DownloadRecord>, String> {
    let mut stmt = conn
        .prepare("SELECT document FROM download_jobs ORDER BY rowid")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    rows.map(|row| {
        serde_json::from_str(&row.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    })
    .collect()
}
pub fn complete(conn: &Connection, record: &DownloadRecord) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    save(&tx, record)?;
    tx.execute("INSERT OR IGNORE INTO downloads(file_name,url,save_path,status,created_at,task_id) VALUES(?1,?2,?3,'Completed',unixepoch(),?4)",params![record.task.file_name,record.task.url,record.task.save_path,record.task.id]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}
