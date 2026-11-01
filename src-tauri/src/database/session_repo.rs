use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::DEFAULT_WORKSPACE_ID;
use rusqlite::{params, Connection};

/// Persists the current open tabs snapshot into SQLite for automatic session restoration.
pub fn save_session_tabs(conn: &Connection, tabs: &[TabInfo]) -> Result<(), String> {
    conn.execute("DELETE FROM session_tabs", [])
        .map_err(|e| e.to_string())?;

    for (idx, tab) in tabs.iter().enumerate() {
        conn.execute(
            "INSERT INTO session_tabs (tab_id, title, url, workspace_id, is_active, is_hibernated, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                tab.id,
                tab.title,
                tab.url,
                tab.workspace_id,
                if tab.is_active { 1 } else { 0 },
                if tab.is_hibernated { 1 } else { 0 },
                idx as i64
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Loads saved session tabs from SQLite. Non-active restored tabs start hibernated to save RAM.
pub fn load_session_tabs(conn: &Connection) -> Result<Vec<TabInfo>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT tab_id, title, url, workspace_id, is_active
             FROM session_tabs
             ORDER BY position ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let is_active: i64 = row.get(4)?;
            let active_bool = is_active != 0;
            let ws_id: String = row
                .get::<_, String>(3)
                .unwrap_or_else(|_| DEFAULT_WORKSPACE_ID.to_string());
            Ok(TabInfo {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                workspace_id: ws_id,
                is_active: active_bool,
                is_hibernated: !active_bool,
                last_active_epoch: TabInfo::current_epoch_secs(),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut restored = Vec::new();
    for item in rows.flatten() {
        restored.push(item);
    }

    if !restored.is_empty() && !restored.iter().any(|t| t.is_active) {
        restored[0].is_active = true;
        restored[0].is_hibernated = false;
    }

    Ok(restored)
}
