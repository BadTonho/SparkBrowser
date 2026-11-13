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
    #[serde(default)]
    pub workspace_isolation: bool,
    pub tab_layout: String,
    pub show_bookmarks_bar: bool,
    pub overlay_blur_px: u8,
    pub workspace_switcher_display: String,
    pub default_search_engine: String,
    pub auto_hibernate_mode: serde_json::Value,
    pub resident_limit: usize,
    #[serde(default)]
    pub hibernation_exemptions: Vec<String>,
    #[serde(default = "default_ask_download_destination")]
    pub ask_download_destination: bool,
    pub language: String,
    pub keyboard_shortcuts: BTreeMap<String, Vec<ShortcutBinding>>,
    #[serde(default)]
    pub site_zoom: Vec<crate::database::site_zoom_repo::SiteZoom>,
    pub update_channel: crate::updates::model::UpdateChannel,
}
fn default_ask_download_destination() -> bool {
    true
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
/// What a restore touches. Workspaces own their tabs and split layouts; bookmarks own their folders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreGroup {
    Workspaces,
    Bookmarks,
    Sessions,
    Notes,
    SiteSearch,
    Settings,
    Shortcuts,
}
impl RestoreGroup {
    pub const ALL: [Self; 7] = [
        Self::Workspaces,
        Self::Bookmarks,
        Self::Sessions,
        Self::Notes,
        Self::SiteSearch,
        Self::Settings,
        Self::Shortcuts,
    ];
    pub const DATA: [Self; 5] = [
        Self::Workspaces,
        Self::Bookmarks,
        Self::Sessions,
        Self::Notes,
        Self::SiteSearch,
    ];
    /// Workspaces stay merge-only: replacing them would tear down live tabs and webviews.
    pub const REPLACEABLE: [Self; 4] = [
        Self::Bookmarks,
        Self::Sessions,
        Self::Notes,
        Self::SiteSearch,
    ];
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreMode {
    #[default]
    Merge,
    Replace,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreOptions {
    pub groups: Vec<RestoreGroup>,
    #[serde(default)]
    pub mode: RestoreMode,
}
impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            groups: RestoreGroup::ALL.to_vec(),
            mode: RestoreMode::Merge,
        }
    }
}
impl RestoreOptions {
    pub fn has(&self, group: RestoreGroup) -> bool {
        self.groups.contains(&group)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.groups.is_empty() {
            return Err("backup.nothing_selected".into());
        }
        let mut seen = std::collections::HashSet::new();
        if !self.groups.iter().all(|group| seen.insert(*group)) {
            return Err("backup.invalid_selection".into());
        }
        Ok(())
    }
    /// Preferences to queue: the backup's values for picked groups, current values for the rest.
    pub fn preferences(
        &self,
        current: &BackupPreferences,
        incoming: &BackupPreferences,
    ) -> Option<BackupPreferences> {
        let (settings, shortcuts) = (
            self.has(RestoreGroup::Settings),
            self.has(RestoreGroup::Shortcuts),
        );
        match (settings, shortcuts) {
            (false, false) => None,
            (true, true) => Some(incoming.clone()),
            (true, false) => {
                let mut mixed = incoming.clone();
                mixed.keyboard_shortcuts = current.keyboard_shortcuts.clone();
                Some(mixed)
            }
            (false, true) => {
                let mut mixed = current.clone();
                mixed.keyboard_shortcuts = incoming.keyboard_shortcuts.clone();
                Some(mixed)
            }
        }
    }
}
#[derive(Clone, Serialize)]
pub struct BackupPreview {
    pub token: String,
    /// Backup creation time (epoch seconds) and its format version.
    pub created_at: i64,
    pub version: u32,
    pub mode: RestoreMode,
    pub groups: Vec<RestoreGroup>,
    pub counts: BTreeMap<String, usize>,
    pub added: BTreeMap<String, usize>,
    /// Local rows that replace mode will delete, per kind.
    pub removed: BTreeMap<String, usize>,
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
    pub removed: BTreeMap<String, usize>,
    /// Groups whose local rows were replaced by this import.
    pub replaced: Vec<RestoreGroup>,
    /// Local history entry holding the pre-replace state, when one was taken.
    pub safety_snapshot: Option<String>,
    pub skipped: Vec<BackupConflict>,
    pub pending_preferences: Option<PreferenceTask>,
    pub snapshot: crate::state::browser_state::TabSnapshot,
    pub workspaces: Vec<WorkspaceItem>,
}
