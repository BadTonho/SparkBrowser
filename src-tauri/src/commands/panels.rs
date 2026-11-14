use tauri::AppHandle;

/// Creates a panel's web view ahead of the click so its first open is as fast as a reopen. Safe to
/// call repeatedly: an existing view is left alone.
#[tauri::command]
pub async fn prepare_panel(app: AppHandle, panel: String) -> Result<(), String> {
    // Creating a web view waits on the main thread, so keep it off the async executor.
    tauri::async_runtime::spawn_blocking(move || crate::window::prepare_panel(&app, &panel))
        .await
        .map_err(|e| e.to_string())?
}
