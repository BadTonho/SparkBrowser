//! Isolated test-app fixtures only. Production never reads this file or grants a test command.
use serde::Deserialize;
use tauri::{AppHandle, Manager};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fixture {
    current_version: String,
    available_version: String,
    #[serde(default)]
    restart_outcome: String,
}
fn fixture(app: &AppHandle) -> Option<Fixture> {
    if app.config().identifier != "org.spark.browser.native-tests" {
        return None;
    }
    let path = app
        .path()
        .app_config_dir()
        .ok()?
        .join("runtime-fixture.json");
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() > 1024 {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}
pub(super) fn apply(app: &AppHandle) -> bool {
    let Some(value) = fixture(app) else {
        return false;
    };
    app.state::<super::RuntimeService>().update(
        app,
        Some(value.current_version),
        Some(value.available_version),
        None,
    );
    true
}
pub(super) fn watch(app: &AppHandle) {
    if app.config().identifier != "org.spark.browser.native-tests" {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            apply(&app);
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    });
}
pub(super) fn restart(app: &AppHandle) -> Option<Result<(), String>> {
    match fixture(app)?.restart_outcome.as_str() {
        "fail" => Some(Err("runtime.restart_failed".into())),
        "handoff" => {
            let result = app
                .path()
                .app_config_dir()
                .map_err(|_| "runtime.restart_failed".to_string())
                .and_then(|path| {
                    std::fs::write(path.join("runtime-restart-requested"), b"restart")
                        .map_err(|_| "runtime.restart_failed".to_string())
                });
            if result.is_ok() {
                app.exit(0);
            }
            Some(result)
        }
        _ => None,
    }
}
