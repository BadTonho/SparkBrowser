//! Storage of the app lock (the secret asked when Spark opens): at most one row in the database of
//! each profile, holding how the profile may be opened and the persistent brake on wrong guesses.
//!
//! Nothing here can read a secret. This table is not part of `BackupData` (so backups, WebDAV sync
//! and Drive never carry it) and is not copied into private-window profiles; a test checks both.
use crate::app_lock::Config;
use rusqlite::{params, Connection, OptionalExtension};

pub fn migrate(conn: &Connection) -> Result<(), String> {
    super::migrations::apply(conn, "app-lock-v1", |conn| {
        conn.execute_batch(
            "CREATE TABLE app_lock(
                id INTEGER PRIMARY KEY CHECK(id=1),
                config TEXT NOT NULL,
                failures INTEGER NOT NULL DEFAULT 0,
                blocked_until INTEGER NOT NULL DEFAULT 0
            );",
        )
        .map_err(|e| e.to_string())
    })
}

/// The lock of this profile, or `None` when it has none.
pub fn load(conn: &Connection) -> Result<Option<Config>, String> {
    let text: Option<String> = conn
        .query_row("SELECT config FROM app_lock WHERE id=1", [], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    text.map(|text| serde_json::from_str(&text).map_err(|_| "lock.corrupt".to_string()))
        .transpose()
}

/// Writes the lock, keeping the brake as it is.
pub fn save(conn: &Connection, config: &Config) -> Result<(), String> {
    let text = serde_json::to_string(config).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO app_lock(id,config) VALUES(1,?1)
         ON CONFLICT(id) DO UPDATE SET config=excluded.config",
        [text],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Removes the lock (and with it the brake).
pub fn clear(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM app_lock", [])
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Wrong guesses in a row, and the unix time before which no attempt is accepted.
pub fn brake(conn: &Connection) -> Result<(u32, i64), String> {
    conn.query_row(
        "SELECT failures, blocked_until FROM app_lock WHERE id=1",
        [],
        |row| Ok((row.get::<_, u32>(0)?, row.get::<_, i64>(1)?)),
    )
    .optional()
    .map(|found| found.unwrap_or((0, 0)))
    .map_err(|e| e.to_string())
}

pub fn set_brake(conn: &Connection, failures: u32, blocked_until: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE app_lock SET failures=?1, blocked_until=?2 WHERE id=1",
        params![failures, blocked_until],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}
