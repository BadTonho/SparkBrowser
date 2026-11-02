use crate::state::{browser_state::BrowserState, diagnostics::Snapshot};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct TabDiagnostic {
    id: String,
    title: String,
    url: String,
    workspace_id: String,
    resident: bool,
    seconds_since_activation: u64,
    protection_reasons: Vec<crate::state::protection::ProtectionReason>,
}

#[derive(Serialize)]
pub struct BrowserDiagnostics {
    startup_ready: bool,
    application_identifier: String,
    application_data_directory: String,
    metrics: Snapshot,
    tabs: usize,
    resident_webviews: usize,
    protected_tabs: usize,
    hibernated_tabs: usize,
    resident_limit: usize,
    tab_details: Vec<TabDiagnostic>,
}

#[tauri::command]
pub async fn get_diagnostics(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<BrowserDiagnostics, String> {
    let _operation = state.tab_operations.lock().await;
    let split = state.current_split();
    let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
    let views = app.webviews();
    let now = crate::state::tab_model::TabInfo::current_epoch_secs();
    let tab_details: Vec<_> = tabs
        .iter()
        .map(|tab| TabDiagnostic {
            id: tab.id.clone(),
            title: tab.title.clone(),
            url: tab.url.clone(),
            workspace_id: tab.workspace_id.clone(),
            resident: views.contains_key(&format!(
                "{}{id}",
                crate::window::webview_manager::CONTENT_WEBVIEW_PREFIX,
                id = tab.id
            )),
            seconds_since_activation: now.saturating_sub(tab.last_active_epoch),
            protection_reasons: tab
                .protection_reasons(split.as_ref().is_some_and(|s| s.contains(&tab.id))),
        })
        .collect();
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
        hibernated_tabs: tab_details.iter().filter(|t| !t.resident).count(),
        resident_limit: state
            .resident_limit
            .load(std::sync::atomic::Ordering::Relaxed),
        tab_details,
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
