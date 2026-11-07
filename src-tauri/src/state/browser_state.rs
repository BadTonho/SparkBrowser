use crate::database::history_repo::record_visit;
use crate::database::session_repo::save_session_tabs;
use crate::privacy::adblock_engine::PrivacyStats;
use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::{WorkspaceItem, DEFAULT_WORKSPACE_ID};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

pub const DEFAULT_HOME_URL: &str = crate::window::new_tab::URL;
pub const INITIAL_TAB_ID: &str = "tab-1";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewportBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for ViewportBounds {
    fn default() -> Self {
        Self {
            x: 276.0,
            y: 46.0,
            width: 996.0,
            height: 766.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TabSnapshot {
    pub revision: u64,
    pub tabs: Vec<TabInfo>,
    pub split_layout: Option<super::split_layout::SplitLayout>,
}

pub struct BrowserState {
    pub startup_ready: AtomicBool,
    pub split_layouts: Mutex<std::collections::HashMap<String, super::split_layout::SplitLayout>>,
    pub split_preview: Mutex<Option<(String, f64)>>,
    pub diagnostics: super::diagnostics::Diagnostics,
    pub resident_limit: std::sync::atomic::AtomicUsize,
    pub hibernation_exemptions: std::sync::Mutex<Vec<String>>,
    pub ask_download_destination: AtomicBool,
    pub snapshot_revision: Mutex<u64>,
    pub tab_operations: tauri::async_runtime::Mutex<()>,
    pub workspaces: Mutex<Vec<WorkspaceItem>>,
    pub active_workspace_id: Mutex<String>,
    pub tabs: Mutex<Vec<TabInfo>>,
    pub active_tab_id: Mutex<String>,
    pub next_tab_counter: Mutex<u64>,
    pub bounds: Mutex<ViewportBounds>,
    pub is_overlay_open: Mutex<bool>,
    pub workspace_alt_pressed: AtomicBool,
    pub caret_browsing_enabled: AtomicBool,
    pub workspace_picker_hold_key: Mutex<String>,
    pub keyboard_shortcuts: Mutex<serde_json::Value>,
    pub keyboard_bridge_sequences: Mutex<crate::security::sequence::Sequences>,
    pub keyboard_bridge_tokens: Mutex<std::collections::HashMap<String, String>>,
    pub workspace_switcher_token: Mutex<Option<String>>,
    pub workspace_switcher_open: Mutex<bool>,
    pub bookmark_context_menu_token: Mutex<Option<String>>,
    pub bookmark_context_menu_open: Mutex<bool>,
    pub bookmark_context_menu_actions: Mutex<Vec<String>>,
    pub shield_script_ids: Mutex<std::collections::HashMap<String, String>>,
    pub shield_enabled: Mutex<bool>,
    pub history_retention_months: std::sync::atomic::AtomicU64,
    pub allow_popups: AtomicBool,
    pub smart_screen_enabled: AtomicBool,
    pub smart_screen_supported: AtomicBool,
    pub auto_skip_ads_enabled: AtomicBool,
    /// Opt-in per-workspace content isolation (own WebView2 data folder each).
    pub workspace_isolation: AtomicBool,
    pub blocked_count: Mutex<u64>,
    pub db: Mutex<Option<Connection>>,
}

impl Default for BrowserState {
    fn default() -> Self {
        let initial_tab = TabInfo::new_default(
            INITIAL_TAB_ID.to_string(),
            DEFAULT_HOME_URL.to_string(),
            true,
        );
        Self {
            startup_ready: AtomicBool::new(false),
            split_layouts: Mutex::new(Default::default()),
            split_preview: Mutex::new(None),
            diagnostics: super::diagnostics::Diagnostics::default(),
            resident_limit: std::sync::atomic::AtomicUsize::new(8),
            hibernation_exemptions: std::sync::Mutex::new(Vec::new()),
            ask_download_destination: AtomicBool::new(true),
            snapshot_revision: Mutex::new(0),
            tab_operations: tauri::async_runtime::Mutex::new(()),
            workspaces: Mutex::new(WorkspaceItem::default_workspaces()),
            active_workspace_id: Mutex::new(DEFAULT_WORKSPACE_ID.to_string()),
            tabs: Mutex::new(vec![initial_tab]),
            active_tab_id: Mutex::new(INITIAL_TAB_ID.to_string()),
            next_tab_counter: Mutex::new(2),
            bounds: Mutex::new(ViewportBounds::default()),
            is_overlay_open: Mutex::new(false),
            workspace_alt_pressed: AtomicBool::new(false),
            caret_browsing_enabled: AtomicBool::new(false),
            workspace_picker_hold_key: Mutex::new("Alt".to_string()),
            keyboard_shortcuts: Mutex::new(serde_json::Value::Object(Default::default())),
            keyboard_bridge_sequences: Mutex::new(Default::default()),
            keyboard_bridge_tokens: Mutex::new(std::collections::HashMap::new()),
            workspace_switcher_token: Mutex::new(None),
            workspace_switcher_open: Mutex::new(false),
            bookmark_context_menu_token: Mutex::new(None),
            bookmark_context_menu_open: Mutex::new(false),
            bookmark_context_menu_actions: Mutex::new(Vec::new()),
            shield_script_ids: Mutex::new(std::collections::HashMap::new()),
            shield_enabled: Mutex::new(true),
            history_retention_months: std::sync::atomic::AtomicU64::new(3),
            allow_popups: AtomicBool::new(false),
            smart_screen_enabled: AtomicBool::new(true),
            smart_screen_supported: AtomicBool::new(false),
            auto_skip_ads_enabled: AtomicBool::new(false),
            workspace_isolation: AtomicBool::new(false),
            blocked_count: Mutex::new(0),
            db: Mutex::new(None),
        }
    }
}

impl BrowserState {
    /// Select a tab and its workspace together, without navigating its WebView.
    pub fn select_tab(&self, tab_id: &str) -> Result<TabInfo, String> {
        let mut active = self.active_tab_id.lock().map_err(|e| e.to_string())?;
        let mut tabs = self.tabs.lock().map_err(|e| e.to_string())?;
        let index = tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or("Tab was not found")?;
        let mut workspace = self.active_workspace_id.lock().map_err(|e| e.to_string())?;
        {
            let mut layouts = self.split_layouts.lock().map_err(|e| e.to_string())?;
            layouts.retain(|_, layout| {
                layout.normalize(&tabs);
                layout.valid(&tabs)
            });
            if let Some(layout) = layouts.get_mut(&tabs[index].workspace_id) {
                layout.focus_tab(tab_id);
            }
        }
        let now = TabInfo::current_epoch_secs();
        for tab in tabs.iter_mut() {
            if tab.id == *active || tab.id == tab_id {
                tab.last_active_epoch = now;
            }
            tab.is_active = tab.id == tab_id;
        }
        tabs[index].is_hibernated = false;
        *active = tab_id.to_string();
        *workspace = tabs[index].workspace_id.clone();
        Ok(tabs[index].clone())
    }

    pub fn move_tab(&self, tab_id: &str, workspace_id: &str) -> Result<(), String> {
        if !self
            .workspaces
            .lock()
            .map_err(|e| e.to_string())?
            .iter()
            .any(|ws| ws.id == workspace_id)
        {
            return Err("Destination workspace was not found".into());
        }
        let active = self.active_tab_id.lock().map_err(|e| e.to_string())?;
        let mut tabs = self.tabs.lock().map_err(|e| e.to_string())?;
        let index = tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or("Tab was not found")?;
        if tabs[index].workspace_id == workspace_id {
            return Ok(());
        }
        let mut layouts = self.split_layouts.lock().map_err(|e| e.to_string())?;
        layouts.retain(|_, layout| {
            layout.normalize(&tabs);
            if layout.workspace_id == tabs[index].workspace_id {
                layout.remove_member(tab_id)
            } else {
                true
            }
        });
        tabs[index].workspace_id = workspace_id.to_string();
        for layout in layouts.values_mut() {
            layout.normalize(&tabs);
        }
        if *active == tab_id {
            if let Some(layout) = layouts.get_mut(workspace_id) {
                layout.focus_tab(tab_id);
            }
            *self.active_workspace_id.lock().map_err(|e| e.to_string())? = workspace_id.to_string();
        }
        Ok(())
    }

    /// Returns a cloned snapshot of all tabs with `is_active` synchronized.
    pub fn snapshot_tabs(&self) -> Vec<TabInfo> {
        let Ok(active_id) = self.active_tab_id.lock() else {
            return Vec::new();
        };

        if let Ok(mut tabs) = self.tabs.lock() {
            for tab in tabs.iter_mut() {
                tab.is_active = tab.id == *active_id;
            }
            return tabs.clone();
        }
        Vec::new()
    }

    /// Saves the current tab list into SQLite for automatic session restoration.
    pub fn snapshot_packet(&self) -> TabSnapshot {
        let mut revision = self
            .snapshot_revision
            .lock()
            .expect("snapshot revision poisoned");
        self.refresh_visible_tabs()
            .expect("visible tab state poisoned");
        let tabs = self.snapshot_tabs();
        *revision += 1;
        TabSnapshot {
            revision: *revision,
            tabs,
            split_layout: self.current_split(),
        }
    }

    pub fn persist_session(&self) -> Result<(), String> {
        self.persist_tab_change(&[], None)
    }

    pub fn persist_tab_change(
        &self,
        closed: &[crate::database::closed_tabs_repo::ClosedTab],
        reopened: Option<i64>,
    ) -> Result<(), String> {
        self.persist_change(closed, reopened, None, None)
    }

    pub fn persist_run_checkpoint(
        &self,
        checkpoint: crate::database::recovery_repo::RunCheckpoint,
    ) -> Result<(), String> {
        self.persist_change(&[], None, Some(checkpoint), None)
    }

    pub fn persist_activated_session(
        &self,
        activation: Option<(&str, bool)>,
    ) -> Result<(), String> {
        self.persist_change(&[], None, None, activation)
    }

    fn persist_change(
        &self,
        closed: &[crate::database::closed_tabs_repo::ClosedTab],
        reopened: Option<i64>,
        checkpoint: Option<crate::database::recovery_repo::RunCheckpoint>,
        activation: Option<(&str, bool)>,
    ) -> Result<(), String> {
        let mut span = self.diagnostics.span("session.persist");
        self.refresh_visible_tabs()?;
        let snapshot = self.snapshot_tabs();
        if checkpoint.is_some()
            && (snapshot.is_empty() || snapshot.iter().filter(|tab| tab.is_active).count() != 1)
        {
            return Err("recovery.invalid_session: Missing active tab".into());
        }
        let layouts = self
            .split_layouts
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let db_guard = self.db.lock().map_err(|e| e.to_string())?;
        let conn = db_guard.as_ref().ok_or("Session database is not ready")?;
        let before = conn.total_changes();
        let result = (|| {
            let transaction = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            save_session_tabs(&transaction, &snapshot)?;
            crate::database::split_repo::save(&transaction, &layouts)?;
            if let Some((tab_id, reset)) = activation {
                if reset {
                    crate::database::tab_usage_repo::reset_tab_history(&transaction, tab_id)?;
                }
                crate::database::tab_usage_repo::record_activation(
                    &transaction,
                    tab_id,
                    TabInfo::current_epoch_secs(),
                )?;
            }
            if !closed.is_empty() {
                crate::database::closed_tabs_repo::append(
                    &transaction,
                    closed,
                    crate::state::tab_model::TabInfo::current_epoch_secs() as i64,
                )?;
            }
            if let Some(id) = reopened {
                crate::database::closed_tabs_repo::consume(&transaction, id)?;
            }
            if let Some(checkpoint) = checkpoint {
                crate::database::recovery_repo::checkpoint(&transaction, checkpoint)?;
            }
            transaction.commit().map_err(|e| e.to_string())
        })();
        span.finish(result.is_ok(), conn.total_changes().saturating_sub(before));
        result
    }

    /// Records a visited URL into the SQLite browsing history table.
    pub fn record_history_visit(&self, title: &str, url: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let retention = crate::database::history_repo::HistoryRetention::from_months(
            self.history_retention_months
                .load(std::sync::atomic::Ordering::Acquire),
        );
        record_visit(
            db.as_ref().ok_or("History database is not ready")?,
            title,
            url,
            retention,
        )
    }

    /// Computes a snapshot of current privacy & RAM hibernation metrics.
    pub fn snapshot_privacy_stats(&self) -> PrivacyStats {
        let shield_enabled = self.shield_enabled.lock().map(|s| *s).unwrap_or(true);
        let blocked_count = self.blocked_count.lock().map(|c| *c).unwrap_or(0);
        let hibernated_tabs_count = self
            .tabs
            .lock()
            .map(|tabs| tabs.iter().filter(|t| t.is_hibernated).count())
            .unwrap_or(0);

        PrivacyStats {
            auto_skip_ads_enabled: self
                .auto_skip_ads_enabled
                .load(std::sync::atomic::Ordering::Acquire),
            shield_enabled,
            blocked_count,
            hibernated_tabs_count,
            supports_live_policy_updates: cfg!(windows),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selecting_a_tab_synchronizes_workspace_and_wakes_only_that_tab() {
        let state = BrowserState::default();
        let mut work = TabInfo::new(
            "tab-2".into(),
            "https://example.test/work".into(),
            "ws-work".into(),
            false,
        );
        work.is_hibernated = true;
        state.tabs.lock().unwrap().push(work);
        let selected = state.select_tab("tab-2").unwrap();
        assert!(!selected.is_hibernated);
        assert_eq!(*state.active_workspace_id.lock().unwrap(), "ws-work");
        let tabs = state.snapshot_tabs();
        assert_eq!(tabs.iter().filter(|tab| tab.is_active).count(), 1);
        assert_eq!(tabs.iter().find(|tab| tab.is_active).unwrap().id, "tab-2");
        assert!(tabs.iter().all(|tab| tab.last_active_epoch > 0));
    }

    #[test]
    fn moving_active_tab_keeps_workspace_selection_consistent() {
        let state = BrowserState::default();
        state.move_tab(INITIAL_TAB_ID, "ws-work").unwrap();
        assert_eq!(*state.active_workspace_id.lock().unwrap(), "ws-work");
        assert_eq!(state.snapshot_tabs()[0].workspace_id, "ws-work");
        assert!(state.move_tab(INITIAL_TAB_ID, "missing").is_err());
        assert!(state.select_tab("missing").is_err());
        assert_eq!(*state.active_tab_id.lock().unwrap(), INITIAL_TAB_ID);
        assert_eq!(*state.active_workspace_id.lock().unwrap(), "ws-work");
    }

    #[test]
    fn session_database_errors_are_reported() {
        assert!(BrowserState::default().persist_session().is_err());
    }
}
