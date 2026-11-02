use super::{browser_state::BrowserState, split_layout::SplitLayout, tab_model::TabInfo};
use std::collections::HashMap;

/// Rollback metadata without closing or rebuilding any existing document.
pub struct TabCheckpoint {
    pub tabs: Vec<TabInfo>,
    pub layouts: HashMap<String, SplitLayout>,
    pub active_tab: String,
    pub active_workspace: String,
}
impl TabCheckpoint {
    pub fn capture(state: &BrowserState) -> Result<Self, String> {
        Ok(Self {
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
    pub fn restore(self, state: &BrowserState) -> Result<(), String> {
        *state.active_tab_id.lock().map_err(|e| e.to_string())? = self.active_tab;
        *state
            .active_workspace_id
            .lock()
            .map_err(|e| e.to_string())? = self.active_workspace;
        *state.tabs.lock().map_err(|e| e.to_string())? = self.tabs;
        *state.split_layouts.lock().map_err(|e| e.to_string())? = self.layouts;
        Ok(())
    }
}
