#[cfg(not(windows))]
pub use super::native_fallback::handler;
#[cfg(windows)]
pub use super::native_windows::{cancel_view, control, install};
#[cfg(not(windows))]
pub async fn control(app: &tauri::AppHandle, id: &str, action: &str) -> Result<(), String> {
    use tauri::Manager;
    let manager = app.state::<super::manager::DownloadManager>();
    if action == "cancel" && !manager.record(id)?.task.pending() {
        manager.update_saved(app, id, |t| t.status = "Cancelled".into())?;
        Ok(())
    } else {
        Err("downloads.native_unavailable".into())
    }
}
#[cfg(not(windows))]
pub fn cancel_view(_: &tauri::AppHandle, _: &str) {}
