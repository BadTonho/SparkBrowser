use super::RecoveryStatus;
use crate::{
    database::{recovery_repo, session_repo, split_repo, workspace_repo},
    state::browser_state::BrowserState,
};
use rusqlite::Connection;
use std::collections::HashSet;

/// No background writer or page WebView may start until this transaction succeeds.
pub fn restore(state: &BrowserState, conn: Connection) -> Result<RecoveryStatus, String> {
    let interrupted = recovery_repo::begin_run(&conn)?;
    let workspaces = workspace_repo::list_workspaces(&conn)?;
    if workspaces.is_empty() {
        return Err("recovery.invalid_session: No workspaces".into());
    }
    let mut tabs = session_repo::load_session_tabs(&conn)?;
    let restored_tabs = tabs.len();
    let layouts = split_repo::load(&conn)?;
    if !tabs.is_empty() {
        let ids: HashSet<_> = tabs.iter().map(|tab| tab.id.as_str()).collect();
        if ids.len() != tabs.len()
            || tabs
                .iter()
                .any(|tab| tab.id.is_empty() || !crate::window::new_tab::valid_tab_url(&tab.url))
        {
            return Err("recovery.invalid_session: Invalid saved tab".into());
        }
        for tab in &mut tabs {
            if !workspaces.iter().any(|ws| ws.id == tab.workspace_id) {
                tab.workspace_id = "ws-personal".into();
            }
        }
        let active = tabs
            .iter()
            .find(|tab| tab.is_active)
            .unwrap_or(&tabs[0])
            .id
            .clone();
        // Older snapshots may contain several active flags. Persist a single consistent selection.
        for tab in &mut tabs {
            tab.is_active = tab.id == active;
            tab.is_visible = tab.is_active;
            tab.is_hibernated = !tab.is_active;
        }
    }
    state.install_restored_session(tabs, workspaces, layouts)?;
    *state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())? = Some(conn);
    // The previous committed snapshot survives any failure, including split or marker writes.
    state.persist_run_checkpoint(recovery_repo::RunCheckpoint::Restored)?;
    Ok(RecoveryStatus {
        recovered: interrupted && restored_tabs > 0,
        restored_tabs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::tab_model::TabInfo;
    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::database::connection::initialize_schema(&conn).unwrap();
        conn
    }
    #[test]
    fn failed_restoration_keeps_all_previous_tabs_and_splits_until_confirmation() {
        let conn = database();
        let mut tabs = vec![
            TabInfo::new_default("a".into(), "https://example.test/a".into(), false),
            TabInfo::new_default("b".into(), "https://example.test/b".into(), true),
        ];
        tabs[0].is_pinned = true;
        session_repo::save_session_tabs(&conn, &tabs).unwrap();
        let layout = crate::state::split_layout::SplitLayout {
            left_tab_ids: Vec::new(),
            right_tab_ids: Vec::new(),
            workspace_id: "ws-personal".into(),
            left_tab_id: "a".into(),
            right_tab_id: "b".into(),
            focused_tab_id: "b".into(),
            ratio: 0.7,
        };
        split_repo::save(
            &conn,
            &std::collections::HashMap::from([("ws-personal".into(), layout.clone())]),
        )
        .unwrap();
        conn.execute_batch("CREATE TRIGGER fail_restore BEFORE UPDATE ON browser_run_state WHEN NEW.restoration_confirmed=1 BEGIN SELECT RAISE(ABORT,'Artificial restore failure'); END;").unwrap();
        let state = BrowserState::default();
        assert!(restore(&state, conn).is_err());
        let mut guard = state.infrastructure().db().lock().unwrap();
        let conn = guard.take().unwrap();
        assert_eq!(
            session_repo::load_session_tabs(&conn)
                .unwrap()
                .iter()
                .map(|tab| tab.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        assert_eq!(split_repo::load(&conn).unwrap()["ws-personal"], layout);
        assert!(!conn
            .query_row(
                "SELECT restoration_confirmed FROM browser_run_state",
                [],
                |row| row.get::<_, bool>(0)
            )
            .unwrap());
        conn.execute_batch("DROP TRIGGER fail_restore").unwrap();
        drop(guard);
        let next = BrowserState::default();
        assert!(restore(&next, conn).unwrap().recovered);
        assert_eq!(
            next.snapshot_tabs()
                .iter()
                .filter(|tab| !tab.is_hibernated)
                .count(),
            2
        );
        assert!(next.snapshot_tabs()[0].is_pinned);
    }
    #[test]
    fn shutdown_persistence_failure_never_marks_a_run_clean() {
        let state = BrowserState::default();
        restore(&state, database()).unwrap();
        state.infrastructure().db().lock().unwrap().as_ref().unwrap().execute_batch("CREATE TRIGGER fail_save BEFORE INSERT ON session_tabs BEGIN SELECT RAISE(ABORT,'Artificial save failure'); END;").unwrap();
        assert!(state
            .persist_run_checkpoint(recovery_repo::RunCheckpoint::CleanExit)
            .is_err());
        assert!(!state
            .infrastructure()
            .db()
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .query_row("SELECT clean_exit FROM browser_run_state", [], |row| row
                .get::<_, bool>(
                0
            ))
            .unwrap());
    }
}
