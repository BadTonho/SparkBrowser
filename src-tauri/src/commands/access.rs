/// Child WebViews cannot invoke application commands, including after navigation to a local URL.
pub fn shell_only(
    handler: impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static,
) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    move |invoke| {
        if invoke.message.webview_ref().label() != "main" {
            invoke
                .resolver
                .reject("Application commands are restricted to the browser shell");
            true
        } else {
            use tauri::Manager;
            if invoke.message.command() != "get_diagnostics"
                && !invoke
                    .message
                    .webview_ref()
                    .state::<crate::state::browser_state::BrowserState>()
                    .startup_ready
                    .load(std::sync::atomic::Ordering::Acquire)
            {
                invoke
                    .resolver
                    .reject("Application startup is still in progress");
                return true;
            }
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
                        | "set_overlay_visibility"
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
