use crate::state::{browser_state::BrowserState, diagnostics::Snapshot};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct BrowserDiagnostics {
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
