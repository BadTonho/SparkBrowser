use super::{browser_state::BrowserState, split_layout::SplitLayout, tab_model::TabInfo};
use std::collections::HashMap;

/// Rollback metadata without closing or rebuilding any existing document.
pub struct TabCheckpoint {
    pub tabs: Vec<TabInfo>,
    pub layouts: HashMap<String, SplitLayout>,
    pub active_tab: String,
    pub active_workspace: String,
    resident_limit: usize,
}
impl TabCheckpoint {
    pub fn capture(state: &BrowserState) -> Result<Self, String> {
        Ok(Self {
            resident_limit: state
                .resident_limit
                .load(std::sync::atomic::Ordering::Relaxed),
            tabs: state.snapshot_tabs(),
            layouts: state
                .split_layouts
                .lock()
                .map_err(|e| e.to_string())?
                .clone(),
            active_tab: state
                .active_tab_id
                .lock()
                .map_err(|e| e.to_string())?
                .clone(),
            active_workspace: state
                .active_workspace_id
                .lock()
                .map_err(|e| e.to_string())?
                .clone(),
        })
    }
    pub fn restore(mut self, state: &BrowserState) -> Result<(), String> {
        state
            .resident_limit
            .store(self.resident_limit, std::sync::atomic::Ordering::Relaxed);
        *state.active_tab_id.lock().map_err(|e| e.to_string())? = self.active_tab;
        *state
            .active_workspace_id
            .lock()
            .map_err(|e| e.to_string())? = self.active_workspace;
        let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
        // Native navigation/media callbacks can finish while persistence is waiting.
        // Roll back organization, while retaining those document observations.
        for tab in &mut self.tabs {
            if let Some(current) = tabs.iter().find(|current| current.id == tab.id) {
                tab.title.clone_from(&current.title);
                tab.url.clone_from(&current.url);
                tab.navigation.clone_from(&current.navigation);
                tab.activity.clone_from(&current.activity);
            }
        }
        *tabs = self.tabs;
        drop(tabs);
        *state.split_layouts.lock().map_err(|e| e.to_string())? = self.layouts;
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
        let before = Self::capture(state)?;
        let result = change().and_then(|value| {
            state.refresh_visible_tabs()?;
            crate::state::memory_policy::enforce_resident_limit(
                &mut state.tabs.lock().map_err(|e| e.to_string())?,
                state
                    .resident_limit
                    .load(std::sync::atomic::Ordering::Relaxed),
            );
            state.persist_activated_session(activation)?;
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
