use super::RecoveryStatus;
use crate::{
    database::{
        recovery_repo, session_repo,
        session_windows_repo::{self, WindowRecord},
        split_repo, workspace_repo,
    },
    state::{
        browser_state::BrowserState,
        tab_model::{TabInfo, MAIN_WINDOW_ID},
    },
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
        assign_windows(
            &mut tabs,
            &session_windows_repo::load_session_windows(&conn)?,
        );
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

/// Tabs of a window that no longer exists move to the main window, and every window keeps exactly
/// one active tab. Older snapshots may contain several active flags; persist one consistent choice.
fn assign_windows(tabs: &mut [TabInfo], windows: &[WindowRecord]) {
    for tab in tabs.iter_mut() {
        if tab.window_id != MAIN_WINDOW_ID
            && !windows.iter().any(|window| window.id == tab.window_id)
        {
            tab.window_id = MAIN_WINDOW_ID.into();
        }
    }
    let mut ids: Vec<String> = Vec::new();
    for tab in tabs.iter() {
        if !ids.contains(&tab.window_id) {
            ids.push(tab.window_id.clone());
        }
    }
    for window in ids {
        let active = tabs
            .iter()
            .filter(|tab| tab.window_id == window)
            .find(|tab| tab.is_active)
            .or_else(|| tabs.iter().find(|tab| tab.window_id == window))
            .map(|tab| tab.id.clone());
        for tab in tabs.iter_mut().filter(|tab| tab.window_id == window) {
            tab.is_active = Some(&tab.id) == active.as_ref();
            tab.is_visible = tab.is_active;
            tab.is_hibernated = !tab.is_active;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tab(id: &str, window: &str, active: bool) -> TabInfo {
        let mut tab = TabInfo::new_default(id.into(), "https://example.test/".into(), active);
        tab.window_id = window.into();
        tab
    }
    fn record(id: &str) -> WindowRecord {
        WindowRecord {
            id: id.into(),
            ..Default::default()
        }
    }

    #[test]
    fn every_window_keeps_one_active_tab_and_unknown_windows_fall_back_to_main() {
        let mut tabs = vec![
            tab("a", "main", true),
            tab("b", "main", true),
            tab("c", "window-2", false),
            tab("d", "window-2", false),
            tab("e", "window-9", true),
        ];
        assign_windows(&mut tabs, &[record("main"), record("window-2")]);
        let active: Vec<_> = tabs
            .iter()
            .filter(|t| t.is_active)
            .map(|t| t.id.as_str())
            .collect();
        // `e` came from a window that is gone, so it joins main (where `a` already won).
        assert_eq!(active, vec!["a", "c"]);
        assert_eq!(tabs[4].window_id, "main");
        assert!(tabs
            .iter()
            .filter(|t| !t.is_active)
            .all(|t| t.is_hibernated));
    }

    #[test]
    fn two_saved_windows_both_restore_with_their_tabs() {
        let conn = database();
        let mut tabs = vec![
            tab("a", "main", true),
            tab("b", "main", false),
            tab("c", "window-2", true),
        ];
        tabs[2].workspace_id = "ws-personal".into();
        session_repo::save_session_tabs(&conn, &tabs).unwrap();
        session_windows_repo::save_session_windows(&conn, &[record("main"), record("window-2")])
            .unwrap();
        let state = BrowserState::default();
        restore(&state, conn).unwrap();
        let restored = state.snapshot_tabs();
        assert_eq!(restored.len(), 3);
        assert_eq!(
            restored
                .iter()
                .filter(|t| t.window_id == "window-2")
                .count(),
            1
        );
        // Which tab is shown per window needs the per-window session state of the next stage;
        // `assign_windows` above already keeps one active tab per window in the saved data.
    }

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
