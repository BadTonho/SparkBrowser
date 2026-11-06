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
            state
                .history_retention_months
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        allow_popups: state
            .allow_popups
            .load(std::sync::atomic::Ordering::Acquire),
        smart_screen_supported: state
            .smart_screen_supported
            .load(std::sync::atomic::Ordering::Acquire),
        smart_screen_enabled: state
            .smart_screen_enabled
            .load(std::sync::atomic::Ordering::Acquire),
    }
}

#[tauri::command]
pub async fn set_history_retention(
    app: AppHandle,
    state: State<'_, BrowserState>,
    retention: HistoryRetention,
) -> Result<(), String> {
    crate::commands::persistence::flush_navigation(app.clone()).await?;
    let _operation = state.tab_operations.lock().await;
    let now = history_repo::current_unix_timestamp();
    executor::run(app, move |conn| {
        crate::database::preferences_repo::store(
            conn,
            crate::database::preferences_repo::HISTORY_RETENTION_KEY,
            match retention {
                HistoryRetention::OneMonth => "one_month",
                HistoryRetention::ThreeMonths => "three_months",
                HistoryRetention::SixMonths => "six_months",
                HistoryRetention::TwelveMonths => "twelve_months",
                HistoryRetention::Always => "always",
            },
        )?;
        history_repo::prune_expired(conn, retention, now)
    })
    .await?;
    state.history_retention_months.store(
        u64::from(retention.months().unwrap_or(0)),
        std::sync::atomic::Ordering::Release,
    );
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
    state
        .allow_popups
        .store(enabled, std::sync::atomic::Ordering::Release);
    Ok(())
}

#[tauri::command]
pub async fn set_smart_screen(
    app: AppHandle,
    state: State<'_, BrowserState>,
    enabled: bool,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    let previous = state
        .smart_screen_enabled
        .load(std::sync::atomic::Ordering::Acquire);
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
    state
        .smart_screen_enabled
        .store(enabled, std::sync::atomic::Ordering::Release);
    Ok(())
}
