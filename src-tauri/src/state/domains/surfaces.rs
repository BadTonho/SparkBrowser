use super::super::browser_state::ViewportBounds;
use crate::database::session_windows_repo::WindowRecord;
use crate::state::tab_model::MAIN_WINDOW_ID;
use crate::sync::Mutex;
use std::{collections::HashMap, sync::atomic::AtomicBool};

/// Domain-owned state. Storage cannot be accessed through BrowserState fields.
pub(crate) struct Surfaces {
    content_profiles: super::content_profiles::ContentProfiles,
    #[cfg(windows)]
    content_bootstrap: super::content_bootstrap::ContentBootstrap,
    /// The page area of each window, in that window's own coordinates.
    bounds: Mutex<HashMap<String, ViewportBounds>>,
    /// Where each window sits on screen, kept so the session can reopen it in the same place.
    geometry: Mutex<HashMap<String, WindowRecord>>,
    is_overlay_open: Mutex<bool>,
    workspace_alt_pressed: AtomicBool,
    workspace_switcher_token: Mutex<Option<String>>,
    workspace_switcher_open: Mutex<bool>,
    bookmark_context_menu_token: Mutex<Option<String>>,
    bookmark_context_menu_open: Mutex<bool>,
    bookmark_context_menu_actions: Mutex<Vec<String>>,
}

impl Surfaces {
    pub(crate) fn content_profiles(&self) -> &super::content_profiles::ContentProfiles {
        &self.content_profiles
    }
    #[cfg(windows)]
    pub(crate) fn content_bootstrap(&self) -> &super::content_bootstrap::ContentBootstrap {
        &self.content_bootstrap
    }
    /// The page area of a window; the default size until its shell reports one.
    pub(crate) fn viewport(&self, window_id: &str) -> Result<ViewportBounds, String> {
        Ok(self
            .bounds
            .lock()
            .map_err(|e| e.to_string())?
            .get(window_id)
            .copied()
            .unwrap_or_default())
    }
    /// Stores a window's page area and says whether it changed.
    pub(crate) fn set_viewport(
        &self,
        window_id: &str,
        bounds: ViewportBounds,
    ) -> Result<bool, String> {
        let mut all = self.bounds.lock().map_err(|e| e.to_string())?;
        Ok(all.insert(window_id.to_string(), bounds) != Some(bounds))
    }
    /// The last known position and size of a window (the active tab fields are not used here).
    pub(crate) fn window_geometry(&self, window_id: &str) -> Option<WindowRecord> {
        self.geometry.lock().ok()?.get(window_id).cloned()
    }
    pub(crate) fn set_window_geometry(&self, window_id: &str, record: WindowRecord) {
        if let Ok(mut all) = self.geometry.lock() {
            all.insert(window_id.to_string(), record);
        }
    }
    pub(crate) fn forget_window_geometry(&self, window_id: &str) {
        if let Ok(mut all) = self.geometry.lock() {
            all.remove(window_id);
        }
    }
    pub(crate) fn forget_viewport(&self, window_id: &str) {
        if let Ok(mut all) = self.bounds.lock() {
            all.remove(window_id);
        }
    }
    /// The main window's page area, for surfaces that have no window of their own yet.
    pub(crate) fn main_viewport(&self) -> Result<ViewportBounds, String> {
        self.viewport(MAIN_WINDOW_ID)
    }
    pub(crate) fn is_overlay_open(&self) -> &Mutex<bool> {
        &self.is_overlay_open
    }
    pub(crate) fn workspace_alt_pressed(&self) -> &AtomicBool {
        &self.workspace_alt_pressed
    }
    pub(crate) fn workspace_switcher_token(&self) -> &Mutex<Option<String>> {
        &self.workspace_switcher_token
    }
    pub(crate) fn workspace_switcher_open(&self) -> &Mutex<bool> {
        &self.workspace_switcher_open
    }
    pub(crate) fn bookmark_context_menu_token(&self) -> &Mutex<Option<String>> {
        &self.bookmark_context_menu_token
    }
    pub(crate) fn bookmark_context_menu_open(&self) -> &Mutex<bool> {
        &self.bookmark_context_menu_open
    }
    pub(crate) fn bookmark_context_menu_actions(&self) -> &Mutex<Vec<String>> {
        &self.bookmark_context_menu_actions
    }
}

impl Default for Surfaces {
    fn default() -> Self {
        Self {
            content_profiles: super::content_profiles::ContentProfiles::default(),
            #[cfg(windows)]
            content_bootstrap: super::content_bootstrap::ContentBootstrap::default(),
            bounds: Mutex::new(HashMap::new()),
            geometry: Mutex::new(HashMap::new()),
            is_overlay_open: Mutex::new(false),
            workspace_alt_pressed: AtomicBool::new(false),
            workspace_switcher_token: Mutex::new(None),
            workspace_switcher_open: Mutex::new(false),
            bookmark_context_menu_token: Mutex::new(None),
            bookmark_context_menu_open: Mutex::new(false),
            bookmark_context_menu_actions: Mutex::new(Vec::new()),
        }
    }
}
