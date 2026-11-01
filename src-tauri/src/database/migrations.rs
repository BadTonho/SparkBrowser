use rusqlite::{Connection, OptionalExtension};

pub fn has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    // Only internal schema identifiers are accepted by callers.
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| e.to_string())?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(columns.iter().any(|name| name == column))
}

/// Schema additions and their marker commit together; failures never masquerade as already applied.
pub fn apply(
    conn: &Connection,
    name: &str,
    migrate: impl FnOnce(&Connection) -> Result<(), String>,
) -> Result<(), String> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS app_migrations (name TEXT PRIMARY KEY)")
        .map_err(|e| e.to_string())?;
    if conn
        .query_row(
            "SELECT 1 FROM app_migrations WHERE name = ?1",
            [name],
            |_| Ok(()),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(());
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    migrate(&tx)?;
    tx.execute("INSERT INTO app_migrations(name) VALUES (?1)", [name])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub fn upgrade_session(conn: &Connection) -> Result<(), String> {
    apply(conn, "session-workspaces-and-pins-v1", |conn| {
        for (column, sql) in [
            ("workspace_id", "ALTER TABLE session_tabs ADD COLUMN workspace_id TEXT NOT NULL DEFAULT 'ws-personal'"),
            ("is_pinned", "ALTER TABLE session_tabs ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0"),
        ] { if !has_column(conn, "session_tabs", column)? { conn.execute_batch(sql).map_err(|e| e.to_string())?; } }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upgrades_legacy_sessions_idempotently_and_rolls_back_failed_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE session_tabs(tab_id TEXT PRIMARY KEY); INSERT INTO session_tabs VALUES ('kept');").unwrap();
        upgrade_session(&conn).unwrap();
        upgrade_session(&conn).unwrap();
        assert!(has_column(&conn, "session_tabs", "workspace_id").unwrap());
        assert_eq!(
            conn.query_row("SELECT tab_id FROM session_tabs", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "kept"
        );
        assert!(apply(&conn, "failure", |tx| {
            tx.execute_batch("CREATE TABLE rollback_me(id)").unwrap();
            Err("failed".into())
        })
        .is_err());
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name='rollback_me'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM app_migrations WHERE name='failure'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
    }
}
