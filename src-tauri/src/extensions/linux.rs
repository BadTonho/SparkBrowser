use tauri::{AppHandle, Webview};

/// Adds an unpacked browser extension on Linux.
///
/// Note: WebKitGTK does not provide native Chromium extension loading out-of-the-box.
/// The detailed blueprint for implementing WebKitWebExtension / WebKitUserContentManager
/// is documented in docs/en/architecture/linux-extensions-guide.md.
pub async fn add_browser_extension(
    _app: &AppHandle,
    _extension_folder_path: &str,
) -> Result<String, String> {
    Err("Browser extensions are not supported on Linux yet".into())
}

/// Toggles extension status on Linux.
pub async fn set_browser_extension_enabled(
    _app: &AppHandle,
    _extension_id: String,
    _enabled: bool,
) -> Result<(), String> {
    Err("Browser extensions are not supported on Linux yet".into())
}

/// Removes extension on Linux.
pub async fn remove_browser_extension(
    _app: &AppHandle,
    _extension_id: String,
) -> Result<(), String> {
    Err("Browser extensions are not supported on Linux yet".into())
}

/// Synchronizes extensions for newly spawned Linux webviews.
pub fn sync_new_webview(_app: &AppHandle, _view: &Webview) -> Result<(), String> {
    Ok(())
}
