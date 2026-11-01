use crate::privacy::adblock_engine::PrivacyStats;
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_privacy_stats(state: State<'_, BrowserState>) -> PrivacyStats {
    state.snapshot_privacy_stats()
}

#[tauri::command]
pub fn toggle_adblock_shield(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<PrivacyStats, String> {
    if let Ok(mut enabled) = state.shield_enabled.lock() {
        *enabled = !*enabled;
    }

    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("privacy-stats-updated", &stats);
    Ok(stats)
}

