use super::{browser_state::BrowserState, split_layout::SplitLayout, tab_model::TabInfo};
use std::collections::HashMap;

/// Rollback metadata without closing or rebuilding any existing document.
pub struct TabCheckpoint {
    pub tabs: Vec<TabInfo>,
    pub layouts: HashMap<String, SplitLayout>,
    /// `(window, active tab, active workspace)` for every open window.
    pub windows: Vec<(String, String, String)>,
    resident_limit: usize,
    resident_limit_enforced: bool,
}
impl TabCheckpoint {
    pub fn capture(state: &BrowserState) -> Result<Self, String> {
        Ok(Self {
            resident_limit: state.session().resident_limit(),
            resident_limit_enforced: state.session().resident_limit_enforced(),
            tabs: state.snapshot_tabs(),
            layouts: state
                .session()
                .edit_split_layouts()
                .lock()
                .map_err(|e| e.to_string())?
                .clone(),
            windows: {
                let mut windows = Vec::new();
                for id in state.session().window_ids() {
                    if let Some(window) = state.session().window(&id) {
                        windows.push((
                            id,
                            window
                                .edit_active_tab_id()
                                .lock()
                                .map_err(|e| e.to_string())?
                                .clone(),
                            window
                                .edit_active_workspace_id()
                                .lock()
                                .map_err(|e| e.to_string())?
                                .clone(),
                        ));
                    }
                }
                windows
            },
        })
    }

    /// The tab a window had selected when the checkpoint was taken.
    pub fn active_tab_of(&self, window_id: &str) -> Option<&str> {
        self.windows
            .iter()
            .find(|(id, _, _)| id == window_id)
            .map(|(_, tab, _)| tab.as_str())
    }
    /// The workspace a window had selected when the checkpoint was taken.
    pub fn active_workspace_of(&self, window_id: &str) -> Option<&str> {
        self.windows
            .iter()
            .find(|(id, _, _)| id == window_id)
            .map(|(_, _, workspace)| workspace.as_str())
    }
    pub fn restore(mut self, state: &BrowserState) -> Result<(), String> {
        state.session().set_resident_limit(self.resident_limit);
        state
            .session()
            .set_resident_limit_enforced(self.resident_limit_enforced);
        // Windows opened by the failed change go away again; windows it closed come back.
        for id in state.session().window_ids() {
            if !self.windows.iter().any(|(window, _, _)| *window == id) {
                state.session().close_window(&id)?;
            }
        }
        for (id, tab, workspace) in &self.windows {
            let window = state
                .session()
                .open_window(id, tab.clone(), workspace.clone())?;
            *window
                .edit_active_tab_id()
                .lock()
                .map_err(|e| e.to_string())? = tab.clone();
            *window
                .edit_active_workspace_id()
                .lock()
                .map_err(|e| e.to_string())? = workspace.clone();
        }
        let mut tabs = state
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?;
        // Native navigation/media callbacks can finish while persistence is waiting.
        // Roll back organization, while retaining those document observations.
        for tab in &mut self.tabs {
            if let Some(current) = tabs.iter().find(|current| current.id == tab.id) {
                if current.navigation.revision > tab.navigation.revision {
                    tab.title.clone_from(&current.title);
                    tab.url.clone_from(&current.url);
                    tab.navigation.clone_from(&current.navigation);
                }
                tab.activity.clone_from(&current.activity);
            }
        }
        *tabs = self.tabs;
        drop(tabs);
        *state
            .session()
            .edit_split_layouts()
            .lock()
            .map_err(|e| e.to_string())? = self.layouts;
        Ok(())
    }

    /// Caller holds tab_operations. Persist before touching native documents.
    pub fn commit<T>(
        state: &BrowserState,
        change: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        Self::commit_with_activation(state, None, change)
    }

    pub fn commit_with_activation<T>(
        state: &BrowserState,
        activation: Option<(&str, bool)>,
        change: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        Self::commit_transaction(state, activation, change, |_| Ok(()))
    }

    pub(in crate::state) fn commit_transaction<T>(
        state: &BrowserState,
        activation: Option<(&str, bool)>,
        change: impl FnOnce() -> Result<T, String>,
        extra: impl FnOnce(&rusqlite::Connection) -> Result<(), String>,
    ) -> Result<T, String> {
        let before = Self::capture(state)?;
        let result = change().and_then(|value| {
            state.refresh_visible_tabs()?;
            crate::state::memory_policy::enforce_resident_limit(
                &mut state
                    .session()
                    .edit_tabs()
                    .lock()
                    .map_err(|e| e.to_string())?,
                state.session().effective_resident_limit(),
                &state
                    .session()
                    .edit_hibernation_exemptions()
                    .lock()
                    .map_err(|e| e.to_string())?,
            );
            state.persist_session_transaction(activation, extra)?;
            Ok(value)
        });
        if result.is_err() {
            before.restore(state)?;
        }
        result
    }
}

#[cfg(test)]
#[path = "tab_checkpoint_tests.rs"]
mod tests;
