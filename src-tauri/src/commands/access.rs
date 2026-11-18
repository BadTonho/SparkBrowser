/// The browser window a shell command came from: the shell webview that invoked it is named after
/// its window. Anything else (an internal overlay, for instance) acts for the main window.
pub fn caller_window(webview: &tauri::Webview) -> String {
    let label = webview.label();
    if crate::state::tab_model::is_window_label(label) {
        label.to_string()
    } else {
        crate::state::tab_model::MAIN_WINDOW_ID.to_string()
    }
}

/// Child WebViews cannot invoke application commands, including after navigation to a local URL.
pub fn shell_only(
    handler: impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static,
) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    move |invoke| {
        use tauri::Manager;
        if !crate::state::tab_model::is_window_label(invoke.message.webview_ref().label())
            || !invoke.message.webview_ref().url().is_ok_and(|url| {
                crate::security::documents::is_shell(
                    invoke.message.webview_ref().app_handle(),
                    &url,
                )
            })
        {
            invoke
                .resolver
                .reject("Application commands are restricted to the browser shell");
            true
        } else {
            if invoke.message.command() != "get_diagnostics"
                && !invoke
                    .message
                    .webview_ref()
                    .state::<crate::state::browser_state::BrowserState>()
                    .infrastructure()
                    .startup_ready()
                    .load(std::sync::atomic::Ordering::Acquire)
            {
                invoke
                    .resolver
                    .reject("Application startup is still in progress");
                return true;
            }
            crate::error_log::trail(&format!("command {}", invoke.message.command()));
            let service = invoke
                .message
                .webview_ref()
                .state::<crate::shutdown::ShutdownService>();
            if service.blocked()
                && !invoke.message.command().starts_with("get_")
                && !matches!(
                    invoke.message.command(),
                    "get_shutdown_state"
                        | "prepare_shutdown"
                        | "confirm_shutdown"
                        | "cancel_shutdown"
                        | "save_page_note"
                        | "push_backup_sync"
                        | "create_local_backup"
                        | "set_overlay_visibility"
                        | "set_tools_popover"
                        | "set_downloads_popover"
                        | "set_suggestions_popover"
                        | "set_commands_popover"
                        | "set_quick_settings_popover"
                        | "set_permission_popover"
                        | "set_history_overlay"
                        | "set_media_popover"
                        | "set_zoom_popover"
                        | "set_tab_drag_preview"
                        | "move_tab_in_split"
                        | "sync_viewport_bounds"
                        | "get_diagnostics"
                        | "get_tabs"
                        | "get_download_tasks"
                        | "get_update_state"
                        | "get_history"
                        | "respond_site_permission"
                )
            {
                invoke.resolver.reject("shutdown.busy");
                return true;
            }
            handler(invoke)
        }
    }
}
