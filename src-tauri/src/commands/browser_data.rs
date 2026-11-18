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
    };
    let outcome = engine_data::clear(&app, wanted).await.inspect_err(|error| {
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
