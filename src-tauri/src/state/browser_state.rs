use crate::database::history_repo::record_visit;
use crate::database::session_repo::save_session_tabs;
use crate::privacy::adblock_engine::PrivacyStats;
use crate::state::tab_model::TabInfo;
use serde::{Deserialize, Serialize};

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

#[derive(Default)]
pub struct BrowserState {
    infrastructure: super::domains::infrastructure::Infrastructure,
    policies: super::domains::policies::Policies,
    security: super::domains::security::Security,
    session: super::domains::session::Session,
    surfaces: super::domains::surfaces::Surfaces,
}

impl BrowserState {
    pub(crate) fn infrastructure(&self) -> &super::domains::infrastructure::Infrastructure {
        &self.infrastructure
    }
    pub(crate) fn policies(&self) -> &super::domains::policies::Policies {
        &self.policies
    }
    pub(crate) fn security(&self) -> &super::domains::security::Security {
        &self.security
    }
    pub(crate) fn session(&self) -> &super::domains::session::Session {
        &self.session
    }
    pub(crate) fn surfaces(&self) -> &super::domains::surfaces::Surfaces {
        &self.surfaces
    }
}

impl BrowserState {
    /// Select a tab and its workspace together, without navigating its WebView.
    pub(in crate::state) fn select_tab(&self, tab_id: &str) -> Result<TabInfo, String> {
        let mut active = self
            .session()
            .edit_active_tab_id()
            .lock()
            .map_err(|e| e.to_string())?;
        let mut tabs = self
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?;
        let index = tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or("Tab was not found")?;
        let mut workspace = self
            .session()
            .edit_active_workspace_id()
            .lock()
            .map_err(|e| e.to_string())?;
        {
            let mut layouts = self
                .session()
                .edit_split_layouts()
                .lock()
                .map_err(|e| e.to_string())?;
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

    pub(in crate::state) fn move_tab(
        &self,
        tab_id: &str,
        workspace_id: &str,
    ) -> Result<(), String> {
        if !self
            .session()
            .edit_workspaces()
            .lock()
            .map_err(|e| e.to_string())?
            .iter()
            .any(|ws| ws.id == workspace_id)
        {
            return Err("Destination workspace was not found".into());
        }
        let active = self
            .session()
            .edit_active_tab_id()
            .lock()
            .map_err(|e| e.to_string())?;
        let mut tabs = self
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?;
        let index = tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or("Tab was not found")?;
        if tabs[index].workspace_id == workspace_id {
            return Ok(());
        }
        let mut layouts = self
            .session()
            .edit_split_layouts()
            .lock()
            .map_err(|e| e.to_string())?;
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
            *self
                .session()
                .edit_active_workspace_id()
                .lock()
                .map_err(|e| e.to_string())? = workspace_id.to_string();
        }
        Ok(())
    }

    /// Returns a cloned snapshot of all tabs with `is_active` synchronized.
    pub fn snapshot_tabs(&self) -> Vec<TabInfo> {
        let Ok(active_id) = self.session().edit_active_tab_id().lock() else {
            return Vec::new();
        };

        if let Ok(mut tabs) = self.session().edit_tabs().lock() {
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
            .session()
            .edit_snapshot_revision()
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
        self.persist_change(closed, reopened, None, None, |_| Ok(()))
    }

    pub fn persist_run_checkpoint(
        &self,
        checkpoint: crate::database::recovery_repo::RunCheckpoint,
    ) -> Result<(), String> {
        self.persist_change(&[], None, Some(checkpoint), None, |_| Ok(()))
    }

    pub fn persist_activated_session(
        &self,
        activation: Option<(&str, bool)>,
    ) -> Result<(), String> {
        self.persist_session_transaction(activation, |_| Ok(()))
    }

    pub(in crate::state) fn persist_session_transaction(
        &self,
        activation: Option<(&str, bool)>,
        extra: impl FnOnce(&rusqlite::Connection) -> Result<(), String>,
    ) -> Result<(), String> {
        self.persist_change(&[], None, None, activation, extra)
    }

    fn persist_change(
        &self,
        closed: &[crate::database::closed_tabs_repo::ClosedTab],
        reopened: Option<i64>,
        checkpoint: Option<crate::database::recovery_repo::RunCheckpoint>,
        activation: Option<(&str, bool)>,
        extra: impl FnOnce(&rusqlite::Connection) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut span = self.infrastructure().diagnostics().span("session.persist");
        self.refresh_visible_tabs()?;
        let snapshot = self.snapshot_tabs();
        if checkpoint.is_some()
            && (snapshot.is_empty() || snapshot.iter().filter(|tab| tab.is_active).count() != 1)
        {
            return Err("recovery.invalid_session: Missing active tab".into());
        }
        let layouts = self
            .session()
            .edit_split_layouts()
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        let db_guard = self
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
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
            extra(&transaction)?;
            transaction.commit().map_err(|e| e.to_string())
        })();
        span.finish(result.is_ok(), conn.total_changes().saturating_sub(before));
        result
    }

    /// Records a visited URL into the SQLite browsing history table.
    pub fn record_history_visit(&self, title: &str, url: &str) -> Result<(), String> {
        let db = self
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let retention = crate::database::history_repo::HistoryRetention::from_months(
            self.policies().history_retention_months(),
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
        let shield_enabled = self
            .policies()
            .shield_enabled()
            .lock()
            .map(|s| *s)
            .unwrap_or(true);
        let blocked_count = self
            .policies()
            .blocked_count()
            .lock()
            .map(|c| *c)
            .unwrap_or(0);
        let hibernated_tabs_count = self
            .session()
            .edit_tabs()
            .lock()
            .map(|tabs| tabs.iter().filter(|t| t.is_hibernated).count())
            .unwrap_or(0);

        PrivacyStats {
            auto_skip_ads_enabled: self.policies().auto_skip_ads_enabled(),
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
        state.session().edit_tabs().lock().unwrap().push(work);
        let selected = state.select_tab("tab-2").unwrap();
        assert!(!selected.is_hibernated);
        assert_eq!(
            *state.session().edit_active_workspace_id().lock().unwrap(),
            "ws-work"
        );
        let tabs = state.snapshot_tabs();
        assert_eq!(tabs.iter().filter(|tab| tab.is_active).count(), 1);
        assert_eq!(tabs.iter().find(|tab| tab.is_active).unwrap().id, "tab-2");
        assert!(tabs.iter().all(|tab| tab.last_active_epoch > 0));
    }

    #[test]
    fn moving_active_tab_keeps_workspace_selection_consistent() {
        let state = BrowserState::default();
        state.move_tab(INITIAL_TAB_ID, "ws-work").unwrap();
        assert_eq!(
            *state.session().edit_active_workspace_id().lock().unwrap(),
            "ws-work"
        );
        assert_eq!(state.snapshot_tabs()[0].workspace_id, "ws-work");
        assert!(state.move_tab(INITIAL_TAB_ID, "missing").is_err());
        assert!(state.select_tab("missing").is_err());
        assert_eq!(
            *state.session().edit_active_tab_id().lock().unwrap(),
            INITIAL_TAB_ID
        );
        assert_eq!(
            *state.session().edit_active_workspace_id().lock().unwrap(),
            "ws-work"
        );
    }

    #[test]
    fn session_database_errors_are_reported() {
        assert!(BrowserState::default().persist_session().is_err());
    }
}
