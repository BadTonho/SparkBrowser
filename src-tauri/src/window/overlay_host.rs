//! Overlays and popovers are single web views (one per kind) that live inside the browser window
//! that opened them. Opening one from another window moves it there by recreating it, and its
//! events go to the shell of the window that hosts it.
use tauri::{AppHandle, Manager, Window};

/// Id of the browser window that hosts the overlay `label` (the main window when it does not exist).
pub fn host_id(app: &AppHandle, label: &str) -> String {
    app.get_webview(label)
        .map(|view| view.window().label().to_string())
        .unwrap_or_else(|| crate::state::tab_model::MAIN_WINDOW_ID.to_string())
}

/// True when the overlay already lives in `window_id`. One that lives in another window is closed,
/// so the caller creates it again where it is needed.
pub fn reuse(app: &AppHandle, label: &str, window_id: &str) -> bool {
    let Some(view) = app.get_webview(label) else {
        return false;
    };
    if view.window().label() == window_id {
        return true;
    }
    let _ = view.close();
    #[cfg(target_os = "linux")]
    super::linux_layout::remove_child(app, label);
    false
}

/// The window an overlay lives in, when it exists.
pub fn window_of(app: &AppHandle, label: &str) -> Option<Window> {
    app.get_webview(label).map(|view| view.window())
}

/// Sends an event to the shell of the window hosting the overlay `label`.
pub fn emit_to_host<S: serde::Serialize + Clone>(
    app: &AppHandle,
    label: &str,
    event: &str,
    payload: S,
) -> tauri::Result<()> {
    crate::security::emit_shell_to(app, &host_id(app, label), event, payload)
}

/// Closes every overlay hosted by a window that is going away, so none keeps the interface blocked.
pub fn release_window(app: &AppHandle, window_id: &str) {
    use super::{
        downloads_popover, history_overlay, media_popover, permission_popover, privacy_popover,
        quick_settings_popover, settings_overlay, tools_popover, zoom_popover,
    };
    let hosted = |label: &str| host_id(app, label) == window_id && app.get_webview(label).is_some();
    if hosted(media_popover::LABEL) {
        media_popover::dismiss(app);
    }
    if hosted(history_overlay::LABEL) {
        history_overlay::dismiss(app);
    }
    if hosted(zoom_popover::LABEL) {
        zoom_popover::dismiss(app);
    }
    if hosted(permission_popover::LABEL) {
        permission_popover::dismiss(app);
    }
    if hosted(privacy_popover::LABEL) {
        privacy_popover::dismiss(app);
    }
    if hosted(super::panel_popover::TOOLS_LABEL) {
        tools_popover::dismiss(app);
    }
    if hosted(super::panel_popover::DOWNLOADS_LABEL) {
        downloads_popover::dismiss(app);
    }
    if hosted(super::panel_popover::QUICK_SETTINGS_LABEL) {
        quick_settings_popover::dismiss(app);
    }
    if hosted(super::panel_popover::SETTINGS_LABEL) {
        settings_overlay::dismiss(app);
    }
    let Some(state) = app.try_state::<crate::state::browser_state::BrowserState>() else {
        return;
    };
    use super::webview_manager::{
        BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, WORKSPACE_SWITCHER_WEBVIEW_LABEL,
    };
    if hosted(WORKSPACE_SWITCHER_WEBVIEW_LABEL) {
        if let Ok(mut open) = state.surfaces().workspace_switcher_open().lock() {
            *open = false;
        }
        if let Ok(mut token) = state.surfaces().workspace_switcher_token().lock() {
            *token = None;
        }
    }
    if hosted(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL) {
        if let Ok(mut open) = state.surfaces().bookmark_context_menu_open().lock() {
            *open = false;
        }
        if let Ok(mut token) = state.surfaces().bookmark_context_menu_token().lock() {
            *token = None;
        }
    }
    // Overlays are children of the window; moving them out keeps their web views alive.
    for label in [
        WORKSPACE_SWITCHER_WEBVIEW_LABEL,
        BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL,
    ] {
        if hosted(label) {
            let _ = reuse(app, label, crate::state::tab_model::MAIN_WINDOW_ID);
        }
    }
}
