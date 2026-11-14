//! Which windows were open, and how, so every one of them can reopen with its own tabs.
use rusqlite::{params, Connection};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowRecord {
    pub id: String,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maximized: bool,
    /// The monitor the window was on; used only where the platform lets windows be placed.
    pub monitor: Option<String>,
    pub active_tab_id: Option<String>,
    pub active_workspace_id: Option<String>,
}

/// Replaces every saved window. Joins an outer transaction when there is one, like the tab writer.
pub fn save_session_windows(conn: &Connection, windows: &[WindowRecord]) -> Result<(), String> {
    if !conn.is_autocommit() {
        return replace(conn, windows);
    }
    let transaction = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    replace(&transaction, windows)?;
    transaction.commit().map_err(|e| e.to_string())
}

fn replace(conn: &Connection, windows: &[WindowRecord]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    if windows
        .iter()
        .any(|window| window.id.is_empty() || !seen.insert(window.id.as_str()))
    {
        return Err("Invalid session window ID".into());
    }
    conn.execute("DELETE FROM session_windows", [])
        .map_err(|e| e.to_string())?;
    let mut insert = conn
        .prepare_cached(
            "INSERT INTO session_windows
             (window_id, position, x, y, width, height, maximized, monitor, active_tab_id, active_workspace_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )
        .map_err(|e| e.to_string())?;
    for (position, window) in windows.iter().enumerate() {
        insert
            .execute(params![
                window.id,
                position as i64,
                window.x,
                window.y,
                window.width,
                window.height,
                window.maximized,
                window.monitor,
                window.active_tab_id,
                window.active_workspace_id,
            ])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn load_session_windows(conn: &Connection) -> Result<Vec<WindowRecord>, String> {
    conn.prepare(
        "SELECT window_id, x, y, width, height, maximized, monitor, active_tab_id, active_workspace_id
         FROM session_windows ORDER BY position ASC",
    )
    .map_err(|e| e.to_string())?
    .query_map([], |row| {
        Ok(WindowRecord {
            id: row.get(0)?,
            x: row.get(1)?,
            y: row.get(2)?,
            width: row.get(3)?,
            height: row.get(4)?,
            maximized: row.get::<_, i64>(5)? != 0,
            monitor: row.get(6)?,
            active_tab_id: row.get(7)?,
            active_workspace_id: row.get(8)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS app_migrations (name TEXT PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS session_tabs (tab_id TEXT PRIMARY KEY);",
        )
        .unwrap();
        crate::database::migrations::upgrade_session_windows(&conn).unwrap();
        conn
    }
    fn window(id: &str, x: i32) -> WindowRecord {
        WindowRecord {
            id: id.into(),
            x: Some(x),
            y: Some(0),
            width: Some(1280),
            height: Some(820),
            maximized: x > 0,
            monitor: Some(format!("monitor-{x}")),
            active_tab_id: Some(format!("tab-{id}")),
            active_workspace_id: Some("ws-personal".into()),
        }
    }

    #[test]
    fn two_windows_round_trip_in_order() {
        let conn = database();
        let windows = vec![window("main", 0), window("window-2", 1920)];
        save_session_windows(&conn, &windows).unwrap();
        assert_eq!(load_session_windows(&conn).unwrap(), windows);
        save_session_windows(&conn, &windows[..1]).unwrap();
        assert_eq!(load_session_windows(&conn).unwrap(), windows[..1]);
    }

    #[test]
    fn invalid_or_failed_saves_keep_the_previous_windows() {
        let conn = database();
        let original = vec![window("main", 0), window("window-2", 1920)];
        save_session_windows(&conn, &original).unwrap();
        assert!(save_session_windows(&conn, &[window("main", 0), window("main", 5)]).is_err());
        assert!(save_session_windows(&conn, &[window("", 0)]).is_err());
        assert_eq!(load_session_windows(&conn).unwrap(), original);
    }

    #[test]
    fn saving_joins_an_outer_transaction() {
        let conn = database();
        save_session_windows(&conn, &[window("main", 0)]).unwrap();
        {
            let tx = conn.unchecked_transaction().unwrap();
            save_session_windows(&tx, &[]).unwrap();
            assert!(load_session_windows(&tx).unwrap().is_empty());
        }
        assert_eq!(load_session_windows(&conn).unwrap().len(), 1);
    }

    #[test]
    fn the_migration_is_idempotent_and_moves_old_tabs_to_main() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE app_migrations (name TEXT PRIMARY KEY);
             CREATE TABLE session_tabs (tab_id TEXT PRIMARY KEY);
             INSERT INTO session_tabs VALUES ('old');",
        )
        .unwrap();
        crate::database::migrations::upgrade_session_windows(&conn).unwrap();
        crate::database::migrations::upgrade_session_windows(&conn).unwrap();
        let window: String = conn
            .query_row("SELECT window_id FROM session_tabs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(window, "main");
        assert!(load_session_windows(&conn).unwrap().is_empty());
    }
}
