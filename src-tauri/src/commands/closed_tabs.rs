use crate::{
    database::closed_tabs_repo::{self, ClosedTab},
    state::{
        browser_state::{BrowserState, TabSnapshot},
        tab_checkpoint::TabCheckpoint,
        tab_model::TabInfo,
    },
    window::webview_manager::sync_all_webviews,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

fn now() -> i64 {
    TabInfo::current_epoch_secs() as i64
}
fn next_id(state: &BrowserState) -> Result<String, String> {
    let mut counter = state.next_tab_counter.lock().map_err(|e| e.to_string())?;
    let id = format!("tab-{}", *counter);
    *counter = counter.checked_add(1).ok_or("closed_tabs.id_limit")?;
    Ok(id)
}
pub fn notify(app: &AppHandle, state: &BrowserState) {
    let rows = state.db.lock().ok().and_then(|db| {
        db.as_ref()
            .and_then(|conn| closed_tabs_repo::list(conn, now()).ok())
    });
    if let Some(rows) = rows {
        let _ = app.emit_to("main", "closed-tabs-updated", rows);
    }
}
pub(super) fn publish(app: &AppHandle, state: &BrowserState) -> TabSnapshot {
    if let Err(error) = sync_all_webviews(app, state) {
        let _ = app.emit_to(
            "main",
            "browser-error",
            format!("The session was saved, but a page could not be displayed: {error}"),
        );
    }
    let snapshot = state.snapshot_packet();
    let _ = app.emit_to("main", "tabs-updated", &snapshot);
    let _ = app.emit_to(
        "main",
        "privacy-stats-updated",
        state.snapshot_privacy_stats(),
    );
    notify(app, state);
    snapshot
}
/// Called with tab_operations held. Reconciliation occurs only after the transaction commits.
pub(super) fn close_one(
    app: &AppHandle,
    state: &BrowserState,
    id: &str,
    _engine: Option<&str>,
) -> Result<TabSnapshot, String> {
    state.refresh_visible_tabs()?;
    let before = TabCheckpoint::capture(state)?;
    if !before.tabs.iter().any(|tab| tab.id == id) {
        return Err("closed_tabs.not_found".into());
    }
    let workspaces = state.workspaces.lock().map_err(|e| e.to_string())?.clone();
    let rows = closed_tabs_repo::records(&before.tabs, &[id.to_string()], &workspaces, now());
    let mut remaining_layouts = before.layouts.clone();
    remaining_layouts.retain(|_, layout| layout.remove_member(id));
    let partner = remaining_layouts
        .get(&before.active_workspace)
        .filter(|_| before.active_tab == id)
        .map(|layout| layout.focused_tab_id.clone())
        .or_else(|| {
            before
                .layouts
                .get(&before.active_workspace)
                .filter(|layout| layout.contains(id))
                .map(|layout| {
                    if layout.left_tab_id == id {
                        layout.right_tab_id.clone()
                    } else {
                        layout.left_tab_id.clone()
                    }
                })
        });
    let result = (|| {
        state
            .tabs
            .lock()
            .map_err(|e| e.to_string())?
            .retain(|tab| tab.id != id);
        *state.split_layouts.lock().map_err(|e| e.to_string())? = remaining_layouts;
        if before.active_tab == id {
            let target = {
                let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
                partner.or_else(|| {
                    tabs.iter()
                        .rev()
                        .find(|tab| tab.workspace_id == before.active_workspace)
                        .map(|tab| tab.id.clone())
                })
            };
            let target = if let Some(target) = target {
                target
            } else {
                let target = next_id(state)?;
                state
                    .tabs
                    .lock()
                    .map_err(|e| e.to_string())?
                    .push(TabInfo::new(
                        target.clone(),
                        crate::window::new_tab::URL.into(),
                        before.active_workspace.clone(),
                        false,
                    ));
                target
            };
            state.select_tab(&target)?;
        }
        state.persist_tab_change(&rows, None)
    })();
    if let Err(error) = result {
        before.restore(state)?;
        return Err(error);
    }
    Ok(publish(app, state))
}

#[tauri::command]
pub async fn get_closed_tabs(state: State<'_, BrowserState>) -> Result<Vec<ClosedTab>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    closed_tabs_repo::list(
        db.as_ref().ok_or("closed_tabs.database_unavailable")?,
        now(),
    )
}
#[tauri::command]
pub async fn clear_closed_tabs(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        closed_tabs_repo::clear(db.as_ref().ok_or("closed_tabs.database_unavailable")?)?;
    }
    notify(&app, &state);
    Ok(())
}
#[derive(Serialize)]
pub struct ReopenClosedTabResult {
    pub snapshot: TabSnapshot,
    pub workspace_missing: bool,
}

pub fn insertion_index(tabs: &[TabInfo], workspace: &str, position: usize) -> usize {
    let positions: Vec<_> = tabs
        .iter()
        .enumerate()
        .filter(|(_, tab)| tab.workspace_id == workspace)
        .map(|(index, _)| index)
        .collect();
    positions
        .get(position)
        .copied()
        .or_else(|| positions.last().map(|index| index + 1))
        .unwrap_or(tabs.len())
}
#[tauri::command]
pub async fn reopen_closed_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    id: Option<i64>,
) -> Result<Option<ReopenClosedTabResult>, String> {
    let _operation = state.tab_operations.lock().await;
    let row = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let rows = closed_tabs_repo::list(
            db.as_ref().ok_or("closed_tabs.database_unavailable")?,
            now(),
        )?;
        if let Some(id) = id {
            Some(
                rows.into_iter()
                    .find(|row| row.id == id)
                    .ok_or("closed_tabs.not_found")?,
            )
        } else {
            rows.into_iter().next()
        }
    };
    let Some(row) = row else {
        return Ok(None);
    };
    if crate::privacy::adblock_engine::should_block_url(
        &row.url,
        *state.shield_enabled.lock().map_err(|e| e.to_string())?,
    ) {
        return Err("closed_tabs.blocked".into());
    }
    if !crate::window::new_tab::valid_tab_url(&row.url) {
        return Err("closed_tabs.invalid_url".into());
    }
    let before = TabCheckpoint::capture(&state)?;
    let workspace_missing = !state
        .workspaces
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .any(|ws| ws.id == row.workspace_id);
    let workspace = if workspace_missing {
        before.active_workspace.clone()
    } else {
        row.workspace_id.clone()
    };
    let id = loop {
        let id = format!(
            "reopened-{}",
            crate::window::webview_manager::create_internal_bridge_token()
        );
        if !before.tabs.iter().any(|tab| tab.id == id) {
            break id;
        }
    };
    let mut tab = TabInfo::new(id.clone(), row.url, workspace.clone(), false);
    tab.title = row.title;
    tab.is_pinned = row.is_pinned;
    let result = (|| {
        let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
        let index = insertion_index(&tabs, &workspace, row.position);
        tabs.insert(index, tab);
        drop(tabs);
        state.select_tab(&id)?;
        state.persist_tab_change(&[], Some(row.id))
    })();
    if let Err(error) = result {
        before.restore(&state)?;
        return Err(error);
    }
    Ok(Some(ReopenClosedTabResult {
        snapshot: publish(&app, &state),
        workspace_missing,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closing_and_reopening_are_atomic_with_session_persistence() {
        let state = BrowserState::default();
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::database::connection::initialize_schema(&conn).unwrap();
        *state.db.lock().unwrap() = Some(conn);
        state.persist_session().unwrap();
        let original = state.snapshot_tabs();
        let records = closed_tabs_repo::records(&original, &[original[0].id.clone()], &[], now());
        state.db.lock().unwrap().as_ref().unwrap().execute_batch("CREATE TRIGGER fail_close BEFORE INSERT ON closed_tabs BEGIN SELECT RAISE(ABORT, 'Artificial failure'); END;").unwrap();
        state.tabs.lock().unwrap().clear();
        assert!(state.persist_tab_change(&records, None).is_err());
        let db = state.db.lock().unwrap();
        let conn = db.as_ref().unwrap();
        assert_eq!(
            crate::database::session_repo::load_session_tabs(conn)
                .unwrap()
                .len(),
            1
        );
        assert!(closed_tabs_repo::list(conn, now()).unwrap().is_empty());
        conn.execute_batch("DROP TRIGGER fail_close").unwrap();
        drop(db);
        state.persist_tab_change(&records, None).unwrap();
        let row = closed_tabs_repo::list(state.db.lock().unwrap().as_ref().unwrap(), now())
            .unwrap()
            .remove(0);
        *state.tabs.lock().unwrap() = original;
        state.db.lock().unwrap().as_ref().unwrap().execute_batch("CREATE TRIGGER fail_reopen BEFORE INSERT ON session_tabs BEGIN SELECT RAISE(ABORT, 'Artificial failure'); END;").unwrap();
        assert!(state.persist_tab_change(&[], Some(row.id)).is_err());
        assert_eq!(
            closed_tabs_repo::list(state.db.lock().unwrap().as_ref().unwrap(), now())
                .unwrap()
                .len(),
            1
        );
    }
    #[test]
    fn reopening_uses_available_positions_inside_the_target_workspace() {
        let tabs = vec![
            TabInfo::new("1".into(), "https://example.test".into(), "a".into(), true),
            TabInfo::new("2".into(), "https://example.test".into(), "b".into(), false),
            TabInfo::new("3".into(), "https://example.test".into(), "a".into(), false),
        ];
        assert_eq!(insertion_index(&tabs, "a", 1), 2);
        assert_eq!(insertion_index(&tabs, "b", 99), 2);
        assert_eq!(insertion_index(&tabs, "missing", 0), 3);
    }
}
