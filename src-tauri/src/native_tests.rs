//! Directory redirection is deliberately unavailable to production binaries.
use std::path::{Path, PathBuf};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};

pub fn configure(context: &mut tauri::Context<tauri::Wry>) -> std::io::Result<()> {
    if context.config().identifier != "org.spark.browser.native-tests" {
        return Ok(());
    }
    let requested = std::env::var_os("SPARK_NATIVE_TEST_RUN_DIR").ok_or_else(|| {
        std::io::Error::other("Native tests require an isolated scenario directory")
    })?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../.cache/native-runs")
        .canonicalize()?;
    let directory = PathBuf::from(requested).canonicalize()?;
    if directory == root || !directory.starts_with(&root) {
        return Err(std::io::Error::other(
            "Native scenario must be inside .cache/native-runs",
        ));
    }
    // EdgeDriver otherwise overrides every WebView's data directory. This runs
    // on the startup thread, before creating the runtime or any WebView, and is
    // available only in the isolated native-test application.
    #[cfg(windows)]
    if std::env::var("SPARK_NATIVE_REAL_PROFILES").as_deref() == Ok("1") {
        std::env::remove_var("WEBVIEW2_USER_DATA_FOLDER");
    }
    // Keep the canonical containment check above, but give native profile APIs
    // a conventional drive path instead of std::fs::canonicalize's verbatim form.
    #[cfg(windows)]
    let directory = directory
        .to_str()
        .and_then(|path| path.strip_prefix(r"\\?\"))
        .filter(|path| path.as_bytes().get(1) == Some(&b':'))
        .map(PathBuf::from)
        .unwrap_or(directory);
    context.config_mut().app.app_directories_override =
        Some(AppDirectoriesOverride::Directories(AppDirectoryOverrides {
            config: Some(directory.join("config")),
            data: Some(directory.join("data")),
            local_data: Some(directory.join("profile")),
            cache: Some(directory.join("cache")),
            log: Some(directory.join("logs")),
        }));
    Ok(())
}
