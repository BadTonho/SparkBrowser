use crate::state::{browser_state::BrowserState, diagnostics::Snapshot};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct BrowserDiagnostics {
    startup_ready: bool,
    application_identifier: String,
    application_data_directory: String,
    metrics: Snapshot,
    tabs: usize,
    resident_webviews: usize,
    protected_tabs: usize,
}

#[tauri::command]
pub async fn get_diagnostics(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<BrowserDiagnostics, String> {
    let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    Ok(BrowserDiagnostics {
        startup_ready: state
            .startup_ready
            .load(std::sync::atomic::Ordering::Acquire),
        application_identifier: app.config().identifier.clone(),
        application_data_directory: app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned(),
        metrics: state.diagnostics.snapshot()?,
        tabs: tabs.len(),
        resident_webviews: app
            .webviews()
            .keys()
            .filter(|label| {
                label.starts_with(crate::window::webview_manager::CONTENT_WEBVIEW_PREFIX)
            })
            .count(),
        protected_tabs: tabs
            .iter()
            .filter(|tab| tab.protected_from_hibernation())
            .count(),
    })
}
