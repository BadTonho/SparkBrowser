use crate::{
    database::{
        bookmarks_repo::BookmarkData, notes_repo::PageNote, saved_sessions_repo::SavedSession,
        site_search_repo::SiteSearchShortcut,
    },
    state::{split_layout::SplitLayout, tab_model::TabInfo, workspace_model::WorkspaceItem},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_BYTES: u64 = 32 * 1024 * 1024;
pub const FORMAT: &str = "veloce-backup";

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupPreferences {
    #[serde(default)]
    pub auto_skip_ads_enabled: bool,
    #[serde(default)]
    pub pip_window_shadow: bool,
    pub tab_layout: String,
    pub show_bookmarks_bar: bool,
    pub overlay_blur_px: u8,
    pub workspace_switcher_display: String,
    pub default_search_engine: String,
    pub auto_hibernate_mode: serde_json::Value,
    pub resident_limit: usize,
    pub language: String,
    pub keyboard_shortcuts: BTreeMap<String, Vec<ShortcutBinding>>,
    #[serde(default)]
    pub site_zoom: Vec<crate::database::site_zoom_repo::SiteZoom>,
    pub update_channel: crate::updates::model::UpdateChannel,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShortcutBinding {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupTab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub workspace_id: String,
    pub is_pinned: bool,
    pub is_active: bool,
}
impl From<&TabInfo> for BackupTab {
    fn from(tab: &TabInfo) -> Self {
        Self {
            id: tab.id.clone(),
            title: tab.title.clone(),
            url: tab.url.clone(),
            workspace_id: tab.workspace_id.clone(),
            is_pinned: tab.is_pinned,
            is_active: tab.is_active,
        }
    }
}
impl BackupTab {
    pub fn tab(&self) -> TabInfo {
        let mut tab = TabInfo::new(
            self.id.clone(),
            self.url.clone(),
            self.workspace_id.clone(),
            self.is_active,
        );
        tab.title = self.title.clone();
        tab.is_pinned = self.is_pinned;
        tab.is_hibernated = !self.is_active;
        tab
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupData {
    pub bookmarks: BookmarkData,
    pub workspaces: Vec<WorkspaceItem>,
    pub sessions: Vec<SavedSession>,
    pub notes: Vec<PageNote>,
    pub site_search: Vec<SiteSearchShortcut>,
    pub tabs: Vec<BackupTab>,
    pub split_layouts: Vec<SplitLayout>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupDocument {
    pub format: String,
    pub version: u32,
    pub created_at: i64,
    pub data: BackupData,
    pub preferences: BackupPreferences,
}
#[derive(Clone, Serialize)]
pub struct BackupConflict {
    pub kind: String,
    pub key: String,
    pub reason: String,
}
#[derive(Clone, Serialize)]
pub struct BackupPreview {
    pub token: String,
    pub counts: BTreeMap<String, usize>,
    pub added: BTreeMap<String, usize>,
    pub skipped: Vec<BackupConflict>,
    pub preferences: BackupPreferences,
}
#[derive(Clone, Serialize)]
pub struct PreferenceTask {
    pub id: String,
    pub preferences: BackupPreferences,
    pub error: Option<String>,
}
#[derive(Serialize)]
pub struct BackupImportResult {
    pub added: BTreeMap<String, usize>,
    pub skipped: Vec<BackupConflict>,
    pub pending_preferences: Option<PreferenceTask>,
    pub snapshot: crate::state::browser_state::TabSnapshot,
    pub workspaces: Vec<WorkspaceItem>,
}
