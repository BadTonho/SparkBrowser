//! Directory redirection is deliberately unavailable to production binaries.
use std::path::{Path, PathBuf};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};

pub fn configure(context: &mut tauri::Context<tauri::Wry>) -> std::io::Result<()> {
    if context.config().identifier != "org.veloce.browser.native-tests" {
        return Ok(());
    }
    let requested = std::env::var_os("VELOCE_NATIVE_TEST_RUN_DIR").ok_or_else(|| {
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
