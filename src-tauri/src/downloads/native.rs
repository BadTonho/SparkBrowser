#[cfg(not(windows))]
pub use super::native_fallback::handler;
#[cfg(windows)]
pub use super::native_windows::{cancel_view, control, install, stop_for_exit};
#[cfg(not(windows))]
pub async fn control(app: &tauri::AppHandle, id: &str, action: &str) -> Result<(), String> {
    use tauri::Manager;
    let manager = app.state::<super::manager::DownloadManager>();
    #[cfg(target_os = "linux")]
    if action == "cancel" {
        let (send, receive) = tokio::sync::oneshot::channel();
        let (handle, task) = (app.clone(), id.to_owned());
        app.run_on_main_thread(move || {
            crate::error_log::ignored(send.send(super::native_linux::cancel(&handle, &task)));
        })
        .map_err(|e| e.to_string())?;
        if receive.await.unwrap_or(false) {
            return Ok(());
        }
    }
    if action == "cancel" && !manager.record(id)?.task.pending() {
        manager.update_saved(app, id, |t| t.status = "Cancelled".into())?;
        Ok(())
    } else {
        Err("downloads.native_unavailable".into())
    }
}
#[cfg(not(windows))]
pub fn cancel_view(_: &tauri::AppHandle, _: &str) {}

#[cfg(not(windows))]
pub fn stop_for_exit(_: &tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "linux")]
pub use super::native_linux::install;
