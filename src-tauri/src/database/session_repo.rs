use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::DEFAULT_WORKSPACE_ID;
use rusqlite::{params, Connection};

/// Persists the current open tabs snapshot into SQLite for automatic session restoration.
pub fn save_session_tabs(conn: &Connection, tabs: &[TabInfo]) -> Result<(), String> {
    // Workspace deletion already owns a transaction. Reuse it rather than
    // opening a nested transaction; standalone saves must also be atomic.
    if !conn.is_autocommit() {
        return replace_session_tabs(conn, tabs);
    }
    let transaction = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    replace_session_tabs(&transaction, tabs)?;
    transaction.commit().map_err(|e| e.to_string())
}

fn replace_session_tabs(conn: &Connection, tabs: &[TabInfo]) -> Result<(), String> {
    let persistent_tabs: Vec<_> = tabs.iter().filter(|tab| !tab.is_ephemeral).collect();
    let ids: std::collections::HashSet<_> =
        persistent_tabs.iter().map(|tab| tab.id.as_str()).collect();
    if ids.len() != persistent_tabs.len() {
        return Err("Duplicate session tab ID".into());
    }
    let existing = conn
        .prepare_cached("SELECT tab_id FROM session_tabs")
        .map_err(|e| e.to_string())?
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut delete = conn
        .prepare_cached("DELETE FROM session_tabs WHERE tab_id = ?1")
        .map_err(|e| e.to_string())?;
    for id in existing {
        if !ids.contains(id.as_str()) {
            delete.execute([id]).map_err(|e| e.to_string())?;
        }
    }
    let mut upsert = conn.prepare_cached(
        "INSERT INTO session_tabs (tab_id, title, url, workspace_id, is_active, is_hibernated, position, is_pinned)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(tab_id) DO UPDATE SET title=excluded.title, url=excluded.url,
           workspace_id=excluded.workspace_id, is_active=excluded.is_active,
           is_hibernated=excluded.is_hibernated, position=excluded.position, is_pinned=excluded.is_pinned
         WHERE title IS NOT excluded.title OR url IS NOT excluded.url
           OR workspace_id IS NOT excluded.workspace_id OR is_active IS NOT excluded.is_active
           OR is_hibernated IS NOT excluded.is_hibernated OR position IS NOT excluded.position OR is_pinned IS NOT excluded.is_pinned"
    ).map_err(|e| e.to_string())?;
    for (idx, tab) in persistent_tabs.iter().enumerate() {
        upsert
            .execute(params![
                tab.id,
                tab.title,
                tab.url,
                tab.workspace_id,
                tab.is_active,
                tab.is_hibernated,
                idx as i64,
                tab.is_pinned
            ])
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Loads saved session tabs from SQLite. Non-active restored tabs start hibernated to save RAM.
pub fn load_session_tabs(conn: &Connection) -> Result<Vec<TabInfo>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT tab_id, title, url, workspace_id, is_active, is_pinned
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
                is_visible: active_bool,
                is_hibernated: !active_bool,
                last_active_epoch: TabInfo::current_epoch_secs(),
                navigation: Default::default(),
                is_pinned: row.get(5)?,
                is_ephemeral: false,
                activity: Default::default(),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut restored = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if !restored.is_empty() && !restored.iter().any(|t| t.is_active) {
        restored[0].is_active = true;
        restored[0].is_hibernated = false;
    }

    Ok(restored)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE session_tabs (
            tab_id TEXT PRIMARY KEY, title TEXT NOT NULL, url TEXT NOT NULL,
            workspace_id TEXT NOT NULL, is_active INTEGER NOT NULL,
            is_hibernated INTEGER NOT NULL, position INTEGER NOT NULL, is_pinned INTEGER NOT NULL DEFAULT 0
        )",
        )
        .unwrap();
        conn
    }

    #[test]
    fn unchanged_sessions_write_no_rows_and_updates_touch_only_changed_tabs() {
        let conn = database();
        let mut tabs = vec![
            TabInfo::new_default("a".into(), "https://example.test/".into(), true),
            TabInfo::new_default("b".into(), "https://example.test/".into(), false),
        ];
        save_session_tabs(&conn, &tabs).unwrap();
        let before = conn.total_changes();
        save_session_tabs(&conn, &tabs).unwrap();
        assert_eq!(conn.total_changes(), before);
        tabs[1].title = "Changed".into();
        tabs[1].is_pinned = true;
        save_session_tabs(&conn, &tabs).unwrap();
        assert_eq!(conn.total_changes(), before + 1);
        assert!(load_session_tabs(&conn).unwrap()[1].is_pinned);
        tabs.pop();
        save_session_tabs(&conn, &tabs).unwrap();
        assert_eq!(conn.total_changes(), before + 2);
    }

    #[test]
    fn failed_save_preserves_the_previous_complete_session() {
        let conn = database();
        let original =
            TabInfo::new_default("original".into(), "https://example.test/".into(), true);
        save_session_tabs(&conn, &[original]).unwrap();
        let duplicate =
            TabInfo::new_default("duplicate".into(), "https://example.test/new".into(), true);
        assert!(save_session_tabs(&conn, &[duplicate.clone(), duplicate]).is_err());
        let restored = load_session_tabs(&conn).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].id, "original");
    }

    #[test]
    fn save_participates_in_an_outer_transaction() {
        let conn = database();
        let tab = TabInfo::new_default("original".into(), "https://example.test/".into(), true);
        save_session_tabs(&conn, &[tab]).unwrap();
        {
            let tx = conn.unchecked_transaction().unwrap();
            save_session_tabs(&tx, &[]).unwrap();
            assert!(load_session_tabs(&tx).unwrap().is_empty());
            // Dropping the outer transaction must restore its session changes.
        }
        assert_eq!(load_session_tabs(&conn).unwrap()[0].id, "original");
    }
}
