use crate::state::workspace_model::DEFAULT_WORKSPACE_ID;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub workspace_id: String,
    pub is_active: bool,
    #[serde(default)]
    pub is_visible: bool,
    pub is_hibernated: bool,
    pub last_active_epoch: u64,
    #[serde(default)]
    pub navigation: NavigationState,
    #[serde(default)]
    pub is_pinned: bool,
    #[serde(default)]
    pub is_ephemeral: bool,
    #[serde(default)]
    pub activity: TabActivity,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TabActivity {
    pub dirty_form: bool,
    pub playing_media: bool,
    pub downloading: bool,
    #[serde(default)]
    pub document_media: bool,
    #[serde(default)]
    pub native_audio: bool,
    #[serde(default)]
    pub frame_media: bool,
    #[serde(default)]
    pub picture_in_picture: bool,
}

impl TabActivity {
    pub fn refresh_media(&mut self) {
        self.playing_media = self.document_media || self.native_audio || self.frame_media;
    }
}

impl TabInfo {
    pub fn protected_from_hibernation(&self) -> bool {
        !self.protection_reasons(false).is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NavigationState {
    pub revision: u64,
    pub loading: bool,
    pub error: Option<String>,
    pub can_go_back: Option<bool>,
    pub can_go_forward: Option<bool>,
}

impl TabInfo {
    pub fn current_epoch_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    pub fn new(id: String, url: String, workspace_id: String, is_active: bool) -> Self {
        let title = derive_title_from_url(&url);
        Self {
            id,
            title,
            url,
            workspace_id,
            is_active,
            is_visible: false,
            is_hibernated: false,
            last_active_epoch: Self::current_epoch_secs(),
            navigation: NavigationState::default(),
            is_pinned: false,
            is_ephemeral: false,
            activity: TabActivity::default(),
        }
    }

    pub fn new_ephemeral(id: String, url: String, workspace_id: String, is_active: bool) -> Self {
        let mut tab = Self::new(id, url, workspace_id, is_active);
        tab.is_ephemeral = true;
        tab
    }

    pub fn new_default(id: String, url: String, is_active: bool) -> Self {
        Self::new(id, url, DEFAULT_WORKSPACE_ID.to_string(), is_active)
    }
}

/// Generates a clean human-readable tab title from a URL hostname or search query.
pub fn derive_title_from_url(raw_url: &str) -> String {
    if let Ok(parsed) = Url::parse(raw_url) {
        if let Some(host) = parsed.host_str() {
            let clean_host = host.trim_start_matches("www.");
            if clean_host.contains("duckduckgo.com") {
                for (k, v) in parsed.query_pairs() {
                    if k == "q" && !v.trim().is_empty() {
                        return format!("{} — Search", v);
                    }
                }
                return "DuckDuckGo".to_string();
            }

            let mut chars = clean_host.chars();
            if let Some(first) = chars.next() {
                return format!("{}{}", first.to_uppercase(), chars.as_str());
            }
            return clean_host.to_string();
        }
    }
    "New Tab".to_string()
}

/// Returns the internal Tauri WebView label for a given tab ID.
pub fn webview_label_for_tab(tab_id: &str) -> String {
    format!("content-webview-{}", tab_id)
}
