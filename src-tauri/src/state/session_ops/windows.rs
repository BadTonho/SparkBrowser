//! Opening and closing browser windows in the session state.
use crate::{
    database::closed_tabs_repo,
    state::{
        browser_state::BrowserState,
        tab_checkpoint::TabCheckpoint,
        tab_model::{TabInfo, MAIN_WINDOW_ID},
    },
};

const WINDOW_PREFIX: &str = "window-";

/// The first unused `window-<n>` id (the first window is always `main`).
pub fn next_window_id(state: &BrowserState) -> String {
    let open = state.session().window_ids();
    (2u32..)
        .map(|n| format!("{WINDOW_PREFIX}{n}"))
        .find(|id| !open.contains(id))
        .expect("an unused window id")
}

/// Registers a window with one new-tab page in `workspace_id`, committed to the session like any tab
/// change. Returns the tab the window starts with.
pub fn open_window(
    state: &BrowserState,
    window_id: &str,
    workspace_id: &str,
) -> Result<String, String> {
    if state.session().window(window_id).is_some() {
        return Err("windows.already_open".into());
    }
    let tab_id = {
        let mut counter = state
            .session()
            .edit_next_tab_counter()
            .lock()
            .map_err(|e| e.to_string())?;
        let id = format!("tab-{}", *counter);
        *counter += 1;
        id
    };
    TabCheckpoint::commit_with_activation(state, Some((&tab_id, true)), || {
        state
            .session()
            .open_window(window_id, tab_id.clone(), workspace_id.to_string())?;
        let mut tab = TabInfo::new(
            tab_id.clone(),
            crate::state::browser_state::DEFAULT_HOME_URL.to_string(),
            workspace_id.to_string(),
            true,
        );
        tab.window_id = window_id.to_string();
        state
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?
            .push(tab);
        state.select_tab(&tab_id)?;
        Ok(())
    })?;
    Ok(tab_id)
}

/// What closing a window removed, so the native pages can follow.
pub struct ClosedWindow {
    pub tab_ids: Vec<String>,
    pub ephemeral_tab_ids: Vec<String>,
}

/// Removes a window and moves its tabs to the recently closed list, where they can be reopened.
/// The main window cannot be closed this way: closing it ends the app.
pub fn close_window(state: &BrowserState, window_id: &str) -> Result<ClosedWindow, String> {
    if window_id == MAIN_WINDOW_ID {
        return Err("windows.cannot_close_main".into());
    }
    let before = TabCheckpoint::capture(state)?;
    let tab_ids: Vec<String> = before
        .tabs
        .iter()
        .filter(|tab| tab.window_id == window_id)
        .map(|tab| tab.id.clone())
        .collect();
    let ephemeral_tab_ids = before
        .tabs
        .iter()
        .filter(|tab| tab.window_id == window_id && tab.is_ephemeral)
        .map(|tab| tab.id.clone())
        .collect();
    let workspaces = state
        .session()
        .edit_workspaces()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let rows = closed_tabs_repo::records(
        &before.tabs,
        &tab_ids,
        &workspaces,
        TabInfo::current_epoch_secs() as i64,
    );
    let result = (|| {
        state
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?
            .retain(|tab| tab.window_id != window_id);
        state
            .session()
            .edit_split_layouts()
            .lock()
            .map_err(|e| e.to_string())?
            .retain(|_, layout| tab_ids.iter().all(|id| layout.remove_member(id)));
        state.session().close_window(window_id)?;
        state.persist_tab_change(&rows, None)
    })();
    if let Err(error) = result {
        before.restore(state)?;
        return Err(error);
    }
    state.surfaces().forget_viewport(window_id);
    Ok(ClosedWindow {
        tab_ids,
        ephemeral_tab_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::browser_state::INITIAL_TAB_ID;

    fn state_with_database() -> BrowserState {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::database::connection::initialize_schema(&conn).unwrap();
        let state = BrowserState::default();
        crate::recovery::startup::restore(&state, conn).unwrap();
        state
    }

    #[test]
    fn a_new_window_starts_with_its_own_tab_and_leaves_main_alone() {
        let state = state_with_database();
        assert_eq!(next_window_id(&state), "window-2");
        let tab = open_window(&state, "window-2", "ws-personal").unwrap();
        assert_eq!(next_window_id(&state), "window-3");
        assert_eq!(state.window_of_tab(&tab).unwrap(), "window-2");
        assert_eq!(
            *state.session().edit_active_tab_id().lock().unwrap(),
            INITIAL_TAB_ID
        );
        let second = state.snapshot_packet_for("window-2");
        assert_eq!(second.tabs.len(), 1);
        assert!(second.tabs[0].is_active);
        assert_eq!(state.snapshot_packet().tabs.len(), 1);
        assert!(open_window(&state, "window-2", "ws-personal").is_err());
    }

    #[test]
    fn closing_a_window_keeps_its_tabs_in_the_recently_closed_list() {
        let state = state_with_database();
        let tab = open_window(&state, "window-2", "ws-personal").unwrap();
        let closed = close_window(&state, "window-2").unwrap();
        assert_eq!(closed.tab_ids, vec![tab]);
        assert!(state.session().window("window-2").is_none());
        assert_eq!(state.snapshot_tabs().len(), 1);
        let rows = closed_tabs_repo::list(
            state
                .infrastructure()
                .db()
                .lock()
                .unwrap()
                .as_ref()
                .unwrap(),
            TabInfo::current_epoch_secs() as i64,
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert!(close_window(&state, MAIN_WINDOW_ID).is_err());
    }

    #[test]
    fn a_failed_open_leaves_no_window_behind() {
        let state = state_with_database();
        state
            .infrastructure()
            .db()
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .execute_batch("CREATE TRIGGER fail_open BEFORE INSERT ON session_tabs WHEN NEW.window_id='window-2' BEGIN SELECT RAISE(ABORT,'open failed'); END;")
            .unwrap();
        assert!(open_window(&state, "window-2", "ws-personal").is_err());
        assert!(state.session().window("window-2").is_none());
        assert_eq!(state.snapshot_tabs().len(), 1);
    }
}
