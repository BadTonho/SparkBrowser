use crate::privacy::adblock_engine::PrivacyStats;
use crate::state::browser_state::BrowserState;
use crate::window::webview_manager::{reset_content_webviews, sync_all_webviews};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_privacy_stats(state: State<'_, BrowserState>) -> PrivacyStats {
    state.snapshot_privacy_stats()
}

#[tauri::command]
pub async fn toggle_adblock_shield(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<PrivacyStats, String> {
    let _operation = state.tab_operations.lock().await;
    if let Ok(mut enabled) = state.shield_enabled.lock() {
        *enabled = !*enabled;
    }

    // Initialization scripts belong to a WebView's lifetime. Recreate views so
    // both the current document and future documents use the new policy.
    reset_content_webviews(&app, &state)?;
    sync_all_webviews(&app, &state)?;
    state.persist_session()?;
    let _ = app.emit("tabs-updated", state.snapshot_tabs());

    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("privacy-stats-updated", &stats);
    Ok(stats)
}
