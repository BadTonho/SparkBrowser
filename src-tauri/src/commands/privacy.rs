use crate::privacy::adblock_engine::PrivacyStats;
use crate::state::browser_state::BrowserState;
#[cfg(not(windows))]
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
    let previous = *state.shield_enabled.lock().map_err(|e| e.to_string())?;
    let enabled = !previous;
    #[cfg(windows)]
    {
        if let Err(error) = crate::window::shield_policy::apply_content_policy(&app, enabled).await
        {
            if crate::window::shield_policy::apply_content_policy(&app, previous)
                .await
                .is_err()
            {
                let _ = app.emit("browser-error", "Could not fully restore the shield policy");
            }
            return Err(error);
        }
        *state.shield_enabled.lock().map_err(|e| e.to_string())? = enabled;
    }
    #[cfg(not(windows))]
    {
        if state
            .tabs
            .lock()
            .map_err(|e| e.to_string())?
            .iter()
            .any(|tab| {
                tab.activity.dirty_form || tab.activity.playing_media || tab.activity.downloading
            })
        {
            return Err("Save edited forms and finish media/downloads before changing the shield on this engine".into());
        }
        *state.shield_enabled.lock().map_err(|e| e.to_string())? = enabled;
        if let Err(error) =
            reset_content_webviews(&app, &state).and_then(|()| sync_all_webviews(&app, &state))
        {
            *state.shield_enabled.lock().map_err(|e| e.to_string())? = previous;
            return Err(error);
        }
        let _ = app.emit(
            "browser-error",
            "This engine rebuilt WebViews to update the shield; page state and history were reset",
        );
    }
    state.persist_session()?;
    let _ = app.emit("tabs-updated", state.snapshot_packet());

    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("privacy-stats-updated", &stats);
    Ok(stats)
}
