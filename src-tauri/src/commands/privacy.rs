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
pub async fn set_auto_skip_ads(
    app: AppHandle,
    state: State<'_, BrowserState>,
    enabled: bool,
) -> Result<PrivacyStats, String> {
    use std::sync::atomic::Ordering;
    let _operation = state.tab_operations.lock().await;
    let previous = state.auto_skip_ads_enabled.load(Ordering::Acquire);
    state
        .auto_skip_ads_enabled
        .store(enabled, Ordering::Release);
    if let Err(error) = crate::privacy::auto_skip::apply(&app, enabled) {
        state
            .auto_skip_ads_enabled
            .store(previous, Ordering::Release);
        if crate::privacy::auto_skip::apply(&app, previous).is_err() {
            let _ = app.emit_to(
                "main",
                "browser-error",
                "Could not fully restore auto skip policy",
            );
        }
        return Err(error);
    }
    let stats = state.snapshot_privacy_stats();
    let _ = app.emit_to("main", "privacy-stats-updated", &stats);
    Ok(stats)
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
        // The live policy does not change session metadata. Fail persistence before
        // touching either current documents or their future-document scripts.
        state.persist_session()?;
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
                tab.activity.dirty_form
                    || (tab.activity.playing_media || tab.activity.picture_in_picture)
                    || tab.activity.downloading
            })
        {
            return Err("Save edited forms and finish media/downloads before changing the shield on this engine".into());
        }
        crate::state::tab_checkpoint::TabCheckpoint::commit(&state, || {
            crate::window::webview_manager::prepare_content_reset(&state, enabled)
        })?;
        *state.shield_enabled.lock().map_err(|e| e.to_string())? = enabled;
        if let Err(error) =
            reset_content_webviews(&app, &state).and_then(|()| sync_all_webviews(&app, &state))
        {
            let _ = app.emit_to(
                "main",
                "browser-error",
                format!("The shield was changed, but some pages could not be rebuilt: {error}"),
            );
        }
        let _ = app.emit(
            "browser-error",
            "This engine rebuilt WebViews to update the shield; page state and history were reset",
        );
    }
    let _ = app.emit("tabs-updated", state.snapshot_packet());

    let stats = state.snapshot_privacy_stats();
    let _ = app.emit("privacy-stats-updated", &stats);
    Ok(stats)
}
