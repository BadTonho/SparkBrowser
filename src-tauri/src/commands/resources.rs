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

/// Puts the pages of the tabs in `low` at the low memory level and the ones in `normal` back at the normal one.
/// Used while a memory limit is on, for the tabs in the background that cannot be hibernated.
#[tauri::command]
pub fn set_tab_memory_levels(app: AppHandle, low: Vec<String>, normal: Vec<String>) -> Result<(), String> {
    let mut failed = Vec::new();
    for (ids, is_low) in [(&low, true), (&normal, false)] {
        for id in ids {
            if let Err(error) = resources::memory_level::set_tab_level(&app, id, is_low) {
                failed.push(format!("{id}: {error}"));
            }
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("Could not set the memory level of: {}", failed.join("; ")))
    }
}
