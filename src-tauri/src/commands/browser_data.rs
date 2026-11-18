//! Clears the web engine's cache and the data sites keep. The history and the download list are cleared by
//! their own commands; this one is only for what the engine stores.
use crate::window::engine_data::{self, Wanted};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClearRequest {
    pub cache: bool,
    pub site_data: bool,
    /// Only what is older than this many days; absent clears everything.
    #[serde(default)]
    pub older_than_days: Option<u32>,
}

#[derive(Serialize)]
pub struct ClearReport {
    /// Stores (data folders) that were cleared.
    pub cleared: usize,
    /// Stores that could not be cleared.
    pub failed: usize,
}

#[tauri::command]
pub async fn clear_browser_data(
    app: AppHandle,
    request: ClearRequest,
) -> Result<ClearReport, String> {
    let wanted = Wanted {
        cache: request.cache,
        site_data: request.site_data,
        older_than_days: request.older_than_days,
    };
    let outcome = engine_data::clear(&app, wanted)
        .await
        .inspect_err(|error| {
            crate::error_log::warn(&format!("Clearing the browser data failed: {error}"));
        })?;
    if outcome.failed > 0 {
        crate::error_log::warn(&format!(
            "Clearing the browser data: {} store(s) cleared, {} failed",
            outcome.cleared, outcome.failed
        ));
    }
    Ok(ClearReport {
        cleared: outcome.cleared,
        failed: outcome.failed,
    })
}

/// What a cleaning would free, in bytes. `cache_bytes` and `site_bytes` are `None` when this system cannot
/// measure them for the chosen age (see `window::engine_usage`).
#[derive(Serialize)]
pub struct CleanupEstimate {
    pub cache_bytes: Option<u64>,
    pub site_bytes: Option<u64>,
    pub history_bytes: u64,
    pub downloads_bytes: u64,
    /// Measuring the engine's folders stopped at its time limit: the real numbers are at least these.
    pub partial: bool,
}

#[tauri::command]
pub async fn estimate_cleanup(
    app: AppHandle,
    older_than_days: Option<u32>,
) -> Result<CleanupEstimate, String> {
    let cutoff = older_than_days.map(crate::database::history_repo::days_ago);
    let (history_bytes, downloads_bytes) =
        crate::database::executor::run(app.clone(), move |conn| {
            Ok((
                crate::database::history_repo::estimate_history_bytes(conn, cutoff)?,
                crate::database::downloads_repo::estimate_list_bytes(conn, cutoff)?,
            ))
        })
        .await?;
    let engine = crate::window::engine_usage::measure(&app, older_than_days)
        .await
        .inspect_err(|error| {
            crate::error_log::warn(&format!("Could not measure the browser data: {error}"));
        })
        .unwrap_or(None);
    Ok(CleanupEstimate {
        cache_bytes: engine.as_ref().map(|usage| usage.cache_bytes),
        site_bytes: engine.as_ref().map(|usage| usage.site_bytes),
        history_bytes,
        downloads_bytes,
        partial: engine.is_some_and(|usage| usage.partial),
    })
}
