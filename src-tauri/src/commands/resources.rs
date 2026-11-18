//! Commands of the resources panel: limits and live numbers. Memory is limited by the interface (it
//! hibernates tabs); the processor and the network are limited here.
use crate::resources::{self, Limits, Usage};
use tauri::AppHandle;

/// `cpu_percent` and `network_kbps` are `0` for "no limit".
#[tauri::command]
pub fn set_resource_limits(app: AppHandle, limits: Limits) -> Result<(), String> {
    resources::configure(&app, limits).inspect_err(|error| {
        crate::error_log::warn(&format!("Could not apply the resource limits: {error}"));
    })
}

/// Measuring the network costs a little, so it runs only while the panel is open.
#[tauri::command]
pub fn set_resource_monitor(app: AppHandle, on: bool) -> Result<(), String> {
    resources::monitor(&app, on)
}

#[tauri::command]
pub fn get_resource_usage(app: AppHandle) -> Usage {
    resources::usage(&app)
}
