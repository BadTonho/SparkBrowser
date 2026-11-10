use crate::{
    database::{
        executor,
        history_repo::{self, HistoryRetention},
    },
    state::browser_state::BrowserState,
};
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct PrivacySettings {
    pub history_retention: HistoryRetention,
    pub allow_popups: bool,
    pub smart_screen_supported: bool,
    pub smart_screen_enabled: bool,
}

#[tauri::command]
pub fn get_privacy_settings(state: State<'_, BrowserState>) -> PrivacySettings {
    PrivacySettings {
        history_retention: HistoryRetention::from_months(
            state.policies().history_retention_months(),
        ),
        allow_popups: state.policies().allow_popups(),
        smart_screen_supported: state.policies().smart_screen_supported(),
        smart_screen_enabled: state.policies().smart_screen_enabled(),
    }
}

#[tauri::command]
pub async fn set_history_retention(
    app: AppHandle,
    state: State<'_, BrowserState>,
    retention: HistoryRetention,
) -> Result<(), String> {
    let _preference = state.policies().history_retention_operation().lock().await;
    // Stop the writer before draining its queue, including visits already queued.
    let old = state.policies().history_retention_months();
    if retention == HistoryRetention::Never {
        state
            .policies()
            .set_history_retention_months(retention.state_value());
    }
    if let Err(error) = crate::commands::persistence::flush_navigation(app.clone()).await {
        state.policies().set_history_retention_months(old);
        return Err(error);
    }
    let _operation = state.session().operation().await;
    let now = history_repo::current_unix_timestamp();
    let result = executor::run(app, move |conn| {
        crate::database::preferences_repo::store(
            conn,
            crate::database::preferences_repo::HISTORY_RETENTION_KEY,
            match retention {
                HistoryRetention::OneMonth => "one_month",
                HistoryRetention::ThreeMonths => "three_months",
                HistoryRetention::SixMonths => "six_months",
                HistoryRetention::TwelveMonths => "twelve_months",
                HistoryRetention::Always => "always",
                HistoryRetention::Never => "never",
            },
        )?;
        history_repo::prune_expired(conn, retention, now)
    })
    .await;
    if let Err(error) = result {
        state.policies().set_history_retention_months(old);
        return Err(error);
    }
    state
        .policies()
        .set_history_retention_months(retention.state_value());
    Ok(())
}

#[tauri::command]
pub async fn set_popup_setting(
    app: AppHandle,
    state: State<'_, BrowserState>,
    enabled: bool,
) -> Result<(), String> {
    executor::run(app, move |conn| {
        crate::database::preferences_repo::store(
            conn,
            crate::database::preferences_repo::ALLOW_POPUPS_KEY,
            if enabled { "true" } else { "false" },
        )
    })
    .await?;
    state.policies().set_allow_popups(enabled);
    Ok(())
}

#[tauri::command]
pub async fn set_smart_screen(
    app: AppHandle,
    state: State<'_, BrowserState>,
    enabled: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    let previous = state.policies().smart_screen_enabled();
    crate::security::set_smart_screen(&app, enabled).await?;
    let persisted = executor::run(app.clone(), move |conn| {
        crate::database::preferences_repo::store(
            conn,
            crate::database::preferences_repo::SMART_SCREEN_KEY,
            if enabled { "true" } else { "false" },
        )
    })
    .await;
    if let Err(error) = persisted {
        if crate::security::set_smart_screen(&app, previous)
            .await
            .is_err()
        {
            let _ = crate::security::emit_shell(
                &app,
                "browser-error",
                "Could not restore SmartScreen settings",
            );
        }
        return Err(error);
    }
    state.policies().set_smart_screen_enabled(enabled);
    Ok(())
}
