use crate::state::{browser_state::BrowserState, tab_model::webview_label_for_tab};
use tauri::{AppHandle, Manager};

/// Shortcut-driven closure transfers keyboard input to the selected page after reconciliation.
/// Resize, background closures and overlay changes never request this transfer.
pub fn focus_active_content_webview(app: &AppHandle, window_id: &str) -> Result<(), String> {
    let window_id = window_id.to_string();
    let id = app
        .state::<BrowserState>()
        .session()
        .window(&window_id)
        .ok_or("windows.not_found")?
        .active_tab_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let app = app.clone();
    let dispatcher = app.clone();
    dispatcher
        .run_on_main_thread(move || {
            let state = app.state::<BrowserState>();
            if !state.session().window(&window_id).is_some_and(|window| {
                window
                    .active_tab_id()
                    .lock()
                    .is_ok_and(|active| *active == id)
            }) || state
                .surfaces()
                .is_overlay_open()
                .lock()
                .map_or(true, |v| *v)
                || state
                    .surfaces()
                    .workspace_switcher_open()
                    .lock()
                    .map_or(true, |v| *v)
                || state
                    .surfaces()
                    .bookmark_context_menu_open()
                    .lock()
                    .map_or(true, |v| *v)
                || super::media_popover::is_open(&app)
                || super::tools_popover::is_open(&app)
                || super::quick_settings_popover::is_open(&app)
                || super::settings_overlay::is_open(&app)
                || super::privacy_popover::is_open(&app)
                || super::permission_popover::is_open(&app)
                || super::history_overlay::is_open(&app)
                || super::zoom_popover::is_open(&app)
                || app.state::<crate::shutdown::ShutdownService>().blocked()
            {
                return;
            }
            if let (Some(window), Some(view)) = (
                app.get_window(&window_id),
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
