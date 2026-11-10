use super::super::browser_state::ViewportBounds;
use std::sync::{atomic::AtomicBool, Mutex};

/// Domain-owned state. Storage cannot be accessed through BrowserState fields.
pub(crate) struct Surfaces {
    content_profiles: super::content_profiles::ContentProfiles,
    #[cfg(windows)]
    content_bootstrap: super::content_bootstrap::ContentBootstrap,
    bounds: Mutex<ViewportBounds>,
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
    pub(crate) fn bounds(&self) -> &Mutex<ViewportBounds> {
        &self.bounds
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
            bounds: Mutex::new(ViewportBounds::default()),
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
