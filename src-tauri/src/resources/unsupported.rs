//! Systems without a way to limit or measure these resources: the limits are accepted and do nothing.
use super::Limits;
use tauri::{AppHandle, Webview};

pub const SUPPORTS_CPU_LIMIT: bool = false;
pub const CPU_LIMIT_IS_PRIORITY: bool = false;
pub const SUPPORTS_NETWORK_LIMIT: bool = false;
pub const SUPPORTS_NETWORK_MONITOR: bool = false;

pub fn apply(_app: &AppHandle, _limits: Limits, _monitoring: bool) -> Result<(), String> {
    Ok(())
}

pub fn attach(_view: &Webview, _limits: Limits, _monitoring: bool) {}

pub fn cpu_seconds() -> Option<f64> {
    None
}
