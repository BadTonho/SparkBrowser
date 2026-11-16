pub mod restart;
mod testing;
#[cfg(windows)]
mod windows;

use serde::Serialize;
use std::sync::Mutex;
use tauri::AppHandle;

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub supported: bool,
    pub current_version: Option<String>,
    pub available_version: Option<String>,
    pub restart_required: bool,
    pub revision: u64,
    pub error: Option<String>,
}
#[derive(Default)]
pub struct RuntimeService(Mutex<RuntimeStatus>);

fn version(value: &str) -> Option<[u32; 4]> {
    let parts: Vec<_> = value
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<_, _>>()
        .ok()?;
    parts.try_into().ok()
}
impl RuntimeService {
    pub fn snapshot(&self) -> Result<RuntimeStatus, String> {
        self.0
            .lock()
            .map(|s| s.clone())
            .map_err(|_| "runtime.unavailable".into())
    }
    pub fn update(
        &self,
        app: &AppHandle,
        current: Option<String>,
        available: Option<String>,
        error: Option<String>,
    ) {
        let Ok(mut state) = self.0.lock() else {
            return;
        };
        let versions = current
            .as_deref()
            .and_then(version)
            .zip(available.as_deref().and_then(version));
        let pending = versions.is_some_and(|(a, b)| b > a);
        let mut next = RuntimeStatus {
            supported: versions.is_some() && error.is_none(),
            current_version: current,
            available_version: available,
            restart_required: pending && error.is_none(),
            revision: state.revision,
            error,
        };
        if *state == next {
            return;
        }
        next.revision += 1;
        *state = next.clone();
        drop(state);
        crate::error_log::ignored(crate::security::emit_shell(
            app,
            "webview-runtime-status",
            next,
        ));
    }
    pub fn ensure_pending(&self) -> Result<(), String> {
        let status = self.snapshot()?;
        if !status.supported {
            return Err("runtime.unavailable".into());
        }
        if status.restart_required {
            Ok(())
        } else {
            Err("runtime.no_update".into())
        }
    }
}

pub fn initialize(app: &AppHandle, view: &tauri::Webview) -> Result<(), String> {
    testing::watch(app);
    #[cfg(windows)]
    windows::install(app, view)?;
    #[cfg(not(windows))]
    let _ = (app, view);
    Ok(())
}

pub fn restart(app: &AppHandle) -> Result<(), String> {
    if let Some(result) = testing::restart(app) {
        return result;
    }
    restart::launch(app)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compare_all_four_version_components_and_reject_unrelated_channels() {
        assert!(version("156.0.4314.59") > version("156.0.4314.58"));
        assert!(version("157.0.1.0") > version("156.99.9999.9999"));
        for text in ["156.0.1", "156.0.1.2 beta", "156.0.1.2.3", "invalid"] {
            assert!(version(text).is_none());
        }
        assert!(!RuntimeService::default().snapshot().unwrap().supported);
        assert!(RuntimeService::default().ensure_pending().is_err());
    }
}
