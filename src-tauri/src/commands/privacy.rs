use crate::privacy::adblock_engine::PrivacyStats;
use crate::state::browser_state::BrowserState;
#[cfg(not(windows))]
use crate::window::webview_manager::{reset_content_webviews, sync_all_webviews};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_shield_exceptions(state: State<'_, BrowserState>) -> Result<Vec<String>, String> {
    state
        .policies()
        .shield_settings()
        .lock()
        .map(|value| value.exceptions.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_shield_exceptions(
    app: AppHandle,
    state: State<'_, BrowserState>,
    exceptions: Vec<String>,
) -> Result<Vec<String>, String> {
    let settings = crate::privacy::tracking_parameters::TrackingSettings {
        enabled: true,
        exceptions,
    }
    .validate()?;
    let _operation = state.session().operation().await;
    if state
        .session()
        .tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .any(|tab| {
            tab.activity.dirty_form
                || tab.activity.playing_media
                || tab.activity.picture_in_picture
                || tab.activity.downloading
        })
    {
        return Err(
            "Save edited forms and finish media/downloads before changing shield exceptions".into(),
        );
    }
    let shield = *state
        .policies()
        .shield_enabled()
        .lock()
        .map_err(|e| e.to_string())?;
    let encoded = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    state.commit_content_policy(
        crate::database::preferences_repo::SHIELD_EXCEPTIONS_KEY,
        &encoded,
        shield,
    )?;
    state.policies().set_shield_settings(settings.clone())?;
    crate::window::webview_manager::reset_content_webviews(&app, &state)?;
    crate::window::webview_manager::sync_all_webviews(&app, &state)?;
    let _ = crate::security::emit_shell(&app, "tabs-updated", state.snapshot_packet());
    Ok(settings.exceptions)
}

#[tauri::command]
pub fn get_gpc(
    state: State<'_, BrowserState>,
) -> Result<crate::privacy::tracking_parameters::TrackingSettings, String> {
    state
        .policies()
        .gpc_settings()
        .lock()
        .map(|value| value.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_gpc(
    app: AppHandle,
    state: State<'_, BrowserState>,
    settings: crate::privacy::tracking_parameters::TrackingSettings,
) -> Result<crate::privacy::tracking_parameters::TrackingSettings, String> {
    let settings = settings.validate()?;
    let _operation = state.session().operation().await;
    if state
        .session()
        .tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .any(|tab| {
            tab.activity.dirty_form
                || tab.activity.playing_media
                || tab.activity.picture_in_picture
                || tab.activity.downloading
        })
    {
        return Err("Save edited forms and finish media/downloads before changing GPC".into());
    }
    let shield = *state
        .policies()
        .shield_enabled()
        .lock()
        .map_err(|e| e.to_string())?;
    let encoded = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    state.commit_content_policy(crate::database::preferences_repo::GPC_KEY, &encoded, shield)?;
    state.policies().set_gpc_settings(settings.clone())?;
    crate::window::webview_manager::reset_content_webviews(&app, &state)?;
    crate::window::webview_manager::sync_all_webviews(&app, &state)?;
    let _ = crate::security::emit_shell(&app, "tabs-updated", state.snapshot_packet());
    Ok(settings)
}

#[tauri::command]
pub fn get_tracking_cleaning(
    state: State<'_, BrowserState>,
) -> Result<crate::privacy::tracking_parameters::TrackingSettings, String> {
    state
        .policies()
        .tracking_settings()
        .lock()
        .map(|value| value.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_tracking_cleaning(
    app: AppHandle,
    state: State<'_, BrowserState>,
    settings: crate::privacy::tracking_parameters::TrackingSettings,
) -> Result<crate::privacy::tracking_parameters::TrackingSettings, String> {
    let settings = settings.validate()?;
    let _operation = state.session().operation().await;
    if state
        .session()
        .tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .any(|tab| {
            tab.activity.dirty_form
                || tab.activity.playing_media
                || tab.activity.picture_in_picture
                || tab.activity.downloading
        })
    {
        return Err(
            "Save edited forms and finish media/downloads before changing link cleaning".into(),
        );
    }
    let shield = *state
        .policies()
        .shield_enabled()
        .lock()
        .map_err(|e| e.to_string())?;
    let encoded = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    state.commit_content_policy(
        crate::database::preferences_repo::TRACKING_CLEANING_KEY,
        &encoded,
        shield,
    )?;
    state.policies().set_tracking_settings(settings.clone())?;
    crate::window::webview_manager::reset_content_webviews(&app, &state)?;
    crate::window::webview_manager::sync_all_webviews(&app, &state)?;
    let _ = crate::security::emit_shell(&app, "tabs-updated", state.snapshot_packet());
    Ok(settings)
}

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
    let _operation = state.session().operation().await;
    let previous = state.policies().auto_skip_ads_enabled();
    state.policies().set_auto_skip_ads_enabled(enabled);
    if let Err(error) = crate::privacy::auto_skip::apply(&app, enabled) {
        state.policies().set_auto_skip_ads_enabled(previous);
        if crate::privacy::auto_skip::apply(&app, previous).is_err() {
            let _ = crate::security::emit_shell(
                &app,
                "browser-error",
                "Could not fully restore auto skip policy",
            );
        }
        return Err(error);
    }
    let stats = state.snapshot_privacy_stats();
    let _ = crate::security::emit_shell(&app, "privacy-stats-updated", &stats);
    Ok(stats)
}

/// Darkens pages that are light. Held in memory only: the interface owns the preference and sends it at
/// start-up and whenever it changes, like the resident-tab limit.
#[tauri::command]
pub fn set_forced_dark_mode(
    app: AppHandle,
    state: State<'_, BrowserState>,
    enabled: bool,
) -> Result<(), String> {
    state.policies().set_forced_dark_enabled(enabled);
    crate::privacy::forced_dark::apply(&app, enabled)
}

#[tauri::command]
pub async fn toggle_adblock_shield(
    app: AppHandle,
    state: State<'_, BrowserState>,
) -> Result<PrivacyStats, String> {
    let _operation = state.session().operation().await;
    let previous = *state
        .policies()
        .shield_enabled()
        .lock()
        .map_err(|e| e.to_string())?;
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
                let _ = crate::security::emit_shell(
                    &app,
                    "browser-error",
                    "Could not fully restore the shield policy",
                );
            }
            return Err(error);
        }
        state.policies().set_shield_enabled(enabled)?;
    }
    #[cfg(not(windows))]
    {
        if state
            .session()
            .tabs()
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
        state
            .policies()
            .set_shield_enabled(enabled)
            .map_err(|e| e.to_string())?;
        if let Err(error) =
            reset_content_webviews(&app, &state).and_then(|()| sync_all_webviews(&app, &state))
        {
            let _ = crate::security::emit_shell(
                &app,
                "browser-error",
                format!("The shield was changed, but some pages could not be rebuilt: {error}"),
            );
        }
        let _ = crate::security::emit_shell(
            &app,
            "browser-error",
            "This engine rebuilt WebViews to update the shield; page state and history were reset",
        );
    }
    let _ = crate::security::emit_shell(&app, "tabs-updated", state.snapshot_packet());

    let stats = state.snapshot_privacy_stats();
    let _ = crate::security::emit_shell(&app, "privacy-stats-updated", &stats);
    Ok(stats)
}
