use super::{
    browser_state::BrowserState, split_layout::SplitLayout, tab_model::TabInfo,
    workspace_model::WorkspaceItem,
};
use std::collections::HashMap;

impl BrowserState {
    /// Backup has already committed its transaction; preserve live observations at handoff.
    pub(crate) fn install_imported_session(
        &self,
        mut tabs: Vec<TabInfo>,
        workspaces: Vec<WorkspaceItem>,
        layouts: &[SplitLayout],
        next_counter: u64,
    ) -> Result<(), String> {
        let mut live = self
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?;
        for tab in &mut tabs {
            if let Some(current) = live.iter().find(|current| current.id == tab.id) {
                if current.navigation.revision > tab.navigation.revision {
                    tab.url.clone_from(&current.url);
                    tab.title.clone_from(&current.title);
                    tab.navigation.clone_from(&current.navigation);
                }
                tab.activity.clone_from(&current.activity);
            }
        }
        *live = tabs;
        drop(live);
        *self
            .session()
            .edit_workspaces()
            .lock()
            .map_err(|e| e.to_string())? = workspaces;
        let mut current = self
            .session()
            .edit_split_layouts()
            .lock()
            .map_err(|e| e.to_string())?;
        for layout in layouts {
            current
                .entry(layout.workspace_id.clone())
                .or_insert_with(|| layout.clone());
        }
        drop(current);
        let mut counter = self
            .session()
            .edit_next_tab_counter()
            .lock()
            .map_err(|e| e.to_string())?;
        *counter = (*counter).max(next_counter);
        Ok(())
    }

    /// Startup-only handoff, before background writers or native documents are started.
    pub(crate) fn install_restored_session(
        &self,
        tabs: Vec<TabInfo>,
        workspaces: Vec<WorkspaceItem>,
        layouts: HashMap<String, SplitLayout>,
        windows: &[crate::database::session_windows_repo::WindowRecord],
    ) -> Result<(), String> {
        let mut others = Vec::new();
        if !tabs.is_empty() {
            // Every saved window comes back with the tab it had selected.
            for tab in tabs.iter().filter(|tab| tab.is_active) {
                if tab.window_id != super::tab_model::MAIN_WINDOW_ID {
                    others.push((
                        tab.window_id.clone(),
                        tab.id.clone(),
                        tab.workspace_id.clone(),
                    ));
                }
            }
            let active = tabs
                .iter()
                .find(|tab| tab.is_active && tab.window_id == super::tab_model::MAIN_WINDOW_ID)
                .ok_or("recovery.invalid_session: Missing active tab")?;
            let next = tabs
                .iter()
                .filter_map(|tab| tab.id.strip_prefix("tab-")?.parse::<u64>().ok())
                .max()
                .unwrap_or(1)
                .checked_add(1)
                .ok_or("recovery.invalid_session: Tab ID limit")?;
            *self
                .session()
                .edit_next_tab_counter()
                .lock()
                .map_err(|e| e.to_string())? = next;
            *self
                .session()
                .edit_active_tab_id()
                .lock()
                .map_err(|e| e.to_string())? = active.id.clone();
            *self
                .session()
                .edit_active_workspace_id()
                .lock()
                .map_err(|e| e.to_string())? = active.workspace_id.clone();
            *self
                .session()
                .edit_tabs()
                .lock()
                .map_err(|e| e.to_string())? = tabs;
            others.sort_by_key(|(id, _, _)| {
                windows
                    .iter()
                    .position(|window| &window.id == id)
                    .unwrap_or(usize::MAX)
            });
            for (id, tab, workspace) in others {
                self.session().open_window(&id, tab, workspace)?;
            }
        }
        for window in windows {
            if self.session().window(&window.id).is_some() {
                self.surfaces()
                    .set_window_geometry(&window.id, window.clone());
            }
        }
        *self
            .session()
            .edit_workspaces()
            .lock()
            .map_err(|e| e.to_string())? = workspaces;
        *self
            .session()
            .edit_split_layouts()
            .lock()
            .map_err(|e| e.to_string())? = layouts;
        self.refresh_visible_tabs()
    }

    pub(crate) fn enforce_resident_policy(&self) -> Result<usize, String> {
        Ok(super::memory_policy::enforce_resident_limit(
            &mut self
                .session()
                .edit_tabs()
                .lock()
                .map_err(|e| e.to_string())?,
            self.session().effective_resident_limit(),
            &self
                .session()
                .hibernation_exemptions()
                .lock()
                .map_err(|e| e.to_string())?,
        ))
    }

    pub(crate) fn prepare_content_reset(&self, enabled: bool) -> Result<(), String> {
        let active = self
            .session()
            .active_tab_id()
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        for tab in self
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?
            .iter_mut()
        {
            tab.is_hibernated = tab.id != active;
            if tab.id == active
                && crate::privacy::adblock_engine::site_is_protected(self, &tab.url)
                && crate::privacy::adblock_engine::should_block_url(&tab.url, enabled)
            {
                tab.url = super::browser_state::DEFAULT_HOME_URL.to_string();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn importing_preserves_document_events_received_after_the_initial_snapshot() {
        let state = BrowserState::default();
        let mut initial = state.snapshot_tabs();
        initial[0].container_id = Some("account".into());
        state
            .observe_document("tab-1", |tab| {
                tab.url = "https://example.test/latest".into();
                tab.activity.native_audio = true;
                tab.activity.refresh_media();
            })
            .unwrap();
        let workspaces = state.session().workspaces().lock().unwrap().clone();
        state
            .install_imported_session(initial, workspaces, &[], 10)
            .unwrap();
        let after = &state.snapshot_tabs()[0];
        assert_eq!(after.url, "https://example.test/latest");
        assert_eq!(after.container_id.as_deref(), Some("account"));
        assert!(after.activity.playing_media);
        assert_eq!(*state.session().next_tab_counter().lock().unwrap(), 10);
    }
}
