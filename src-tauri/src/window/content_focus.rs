use crate::state::{browser_state::BrowserState, tab_model::webview_label_for_tab};
use tauri::{AppHandle, Manager};

/// Shortcut-driven closure transfers keyboard input to the selected page after reconciliation.
/// Resize, background closures and overlay changes never request this transfer.
pub fn focus_active_content_webview(app: &AppHandle) -> Result<(), String> {
    let id = app
        .state::<BrowserState>()
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let app = app.clone();
    let dispatcher = app.clone();
    dispatcher
        .run_on_main_thread(move || {
            let state = app.state::<BrowserState>();
            if !state.active_tab_id.lock().is_ok_and(|active| *active == id)
                || state.is_overlay_open.lock().map_or(true, |v| *v)
                || state.workspace_switcher_open.lock().map_or(true, |v| *v)
                || state.bookmark_context_menu_open.lock().map_or(true, |v| *v)
                || super::media_popover::is_open(&app)
                || super::privacy_popover::is_open(&app)
                || app.state::<crate::shutdown::ShutdownService>().blocked()
            {
                return;
            }
            if let (Some(window), Some(view)) = (
                app.get_window("main"),
                app.get_webview(&webview_label_for_tab(&id)),
            ) {
                if !window.is_minimized().unwrap_or(true) {
                    let _ = window.set_focus();
                    let _ = view.set_focus();
                }
            }
        })
        .map_err(|e| e.to_string())
}
