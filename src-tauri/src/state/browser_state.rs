use crate::database::history_repo::record_visit;
use crate::database::session_repo::save_session_tabs;
use crate::privacy::adblock_engine::PrivacyStats;
use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::{WorkspaceItem, DEFAULT_WORKSPACE_ID};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

pub const DEFAULT_HOME_URL: &str = "https://www.google.com/";
pub const INITIAL_TAB_ID: &str = "tab-1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

pub struct BrowserState {
    pub workspaces: Mutex<Vec<WorkspaceItem>>,
    pub active_workspace_id: Mutex<String>,
    pub tabs: Mutex<Vec<TabInfo>>,
    pub active_tab_id: Mutex<String>,
    pub next_tab_counter: Mutex<u64>,
    pub bounds: Mutex<ViewportBounds>,
    pub is_overlay_open: Mutex<bool>,
    pub workspace_alt_pressed: AtomicBool,
    pub workspace_picker_hold_key: Mutex<String>,
    pub keyboard_shortcuts: Mutex<serde_json::Value>,
    pub keyboard_bridge_token: Mutex<Option<String>>,
    pub workspace_switcher_token: Mutex<Option<String>>,
    pub workspace_switcher_open: Mutex<bool>,
    pub shield_enabled: Mutex<bool>,
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
            workspaces: Mutex::new(WorkspaceItem::default_workspaces()),
            active_workspace_id: Mutex::new(DEFAULT_WORKSPACE_ID.to_string()),
            tabs: Mutex::new(vec![initial_tab]),
            active_tab_id: Mutex::new(INITIAL_TAB_ID.to_string()),
            next_tab_counter: Mutex::new(2),
            bounds: Mutex::new(ViewportBounds::default()),
            is_overlay_open: Mutex::new(false),
            workspace_alt_pressed: AtomicBool::new(false),
            workspace_picker_hold_key: Mutex::new("Alt".to_string()),
            keyboard_shortcuts: Mutex::new(serde_json::Value::Object(Default::default())),
            keyboard_bridge_token: Mutex::new(None),
            workspace_switcher_token: Mutex::new(None),
            workspace_switcher_open: Mutex::new(false),
            shield_enabled: Mutex::new(true),
            blocked_count: Mutex::new(0),
            db: Mutex::new(None),
        }
    }
}

impl BrowserState {
    /// Returns a cloned snapshot of all tabs with `is_active` synchronized.
    pub fn snapshot_tabs(&self) -> Vec<TabInfo> {
        let active_id = self
            .active_tab_id
            .lock()
            .map(|id| id.clone())
            .unwrap_or_default();

        if let Ok(mut tabs) = self.tabs.lock() {
            for tab in tabs.iter_mut() {
                tab.is_active = tab.id == active_id;
            }
            return tabs.clone();
        }
        Vec::new()
    }

    /// Saves the current tab list into SQLite for automatic session restoration.
    pub fn persist_session(&self) {
        let snapshot = self.snapshot_tabs();
        if let Ok(db_guard) = self.db.lock() {
            if let Some(conn) = db_guard.as_ref() {
                let _ = save_session_tabs(conn, &snapshot);
            }
        }
    }

    /// Records a visited URL into the SQLite browsing history table.
    pub fn record_history_visit(&self, title: &str, url: &str) {
        if let Ok(db_guard) = self.db.lock() {
            if let Some(conn) = db_guard.as_ref() {
                let _ = record_visit(conn, title, url);
            }
        }
    }

    /// Records a local activation event without storing page URLs or titles.
    pub fn record_tab_activation(&self, tab_id: &str) {
        if let Ok(db_guard) = self.db.lock() {
            if let Some(conn) = db_guard.as_ref() {
                let _ = crate::database::tab_usage_repo::record_activation(
                    conn,
                    tab_id,
                    TabInfo::current_epoch_secs(),
                );
            }
        }
    }

    /// Starts usage tracking for a newly created tab, clearing stale events if its ID was reused.
    pub fn record_new_tab_activation(&self, tab_id: &str) {
        if let Ok(db_guard) = self.db.lock() {
            if let Some(conn) = db_guard.as_ref() {
                let _ = crate::database::tab_usage_repo::reset_tab_history(conn, tab_id);
                let _ = crate::database::tab_usage_repo::record_activation(
                    conn,
                    tab_id,
                    TabInfo::current_epoch_secs(),
                );
            }
        }
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
            shield_enabled,
            blocked_count,
            hibernated_tabs_count,
        }
    }
}
