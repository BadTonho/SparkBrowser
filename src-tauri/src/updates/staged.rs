//! A verified update downloaded in the background, kept on disk so that the next start can install it.
//!
//! Nothing here trusts the files: the installer is verified again against the signature in the saved
//! manifest (bound to the version, see `requireSignedVersion`) right before it runs, and an update that
//! is not newer than the running application is never installed.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const META: &str = "update.json";
const INSTALLER: &str = "installer.bin";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Staged {
    pub version: String,
    pub channel: String,
    /// The release manifest the updater was restored from (it carries the signature).
    pub manifest: serde_json::Value,
}

pub fn directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("updates").join("staged"))
        .map_err(|e| e.to_string())
}

/// Replaces any earlier staged update. The metadata is renamed last, so a half-written update has no
/// metadata and is ignored.
pub fn write(dir: &Path, staged: &Staged, installer: &[u8]) -> Result<(), String> {
    clear(dir);
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let installer_tmp = dir.join(format!("{INSTALLER}.tmp"));
    let meta_tmp = dir.join(format!("{META}.tmp"));
    std::fs::write(&installer_tmp, installer).map_err(|e| e.to_string())?;
    std::fs::write(
        &meta_tmp,
        serde_json::to_vec(staged).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(&installer_tmp, dir.join(INSTALLER)).map_err(|e| e.to_string())?;
    std::fs::rename(&meta_tmp, dir.join(META)).map_err(|e| e.to_string())
}

pub fn read(dir: &Path) -> Option<(Staged, Vec<u8>)> {
    let staged: Staged = serde_json::from_slice(&std::fs::read(dir.join(META)).ok()?).ok()?;
    let installer = std::fs::read(dir.join(INSTALLER)).ok()?;
    (!installer.is_empty()).then_some((staged, installer))
}

pub fn clear(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

pub fn clear_for(app: &tauri::AppHandle) {
    if let Ok(dir) = directory(app) {
        clear(&dir);
    }
}

/// Whether a staged update may still be installed by a running application of `current`.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn installable(staged: &Staged, current: &str, channel: &str, auto_update: bool) -> bool {
    let (Ok(staged_version), Ok(current)) = (
        semver::Version::parse(&staged.version),
        semver::Version::parse(current),
    ) else {
        return false;
    };
    auto_update && staged.channel == channel && staged_version > current
}

/// Installs the staged update, if there is a valid one. Called before the session is restored, so the
/// installer's restart leaves the clean-exit marker of the previous run untouched. The staged files are
/// removed first: a failing installer must never be retried at every start. Returns only when nothing
/// was installed.
#[cfg(windows)]
pub fn install_pending(
    app: &tauri::AppHandle,
    current: &str,
    channel: super::model::UpdateChannel,
    auto_update: bool,
) -> Result<(), String> {
    use tauri::Manager;
    use tauri_plugin_updater::UpdaterExt;
    let dir = directory(app)?;
    let Some((staged, installer)) = read(&dir) else {
        clear(&dir);
        return Ok(());
    };
    clear(&dir);
    if !installable(&staged, current, channel.as_str(), auto_update) {
        return Ok(());
    }
    // The endpoint is required by the builder but never contacted: the update is restored from the manifest.
    let endpoint = super::releases::asset_url(&format!("v{}", staged.version), "updater.json");
    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint.parse().map_err(|e| format!("{e}"))?])
        .map_err(|e| e.to_string())?
        .target(super::releases::TARGET)
        .build()
        .map_err(|e| e.to_string())?;
    let update = updater
        .restore_update(staged.manifest)
        .map_err(|e| e.to_string())?
        .ok_or("The staged update is not newer than this application")?;
    if update.version != staged.version {
        return Err("The staged update does not match its manifest".into());
    }
    update
        .verify(&installer)
        .map_err(|_| "Staged update signature verification failed".to_string())?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    update.install(&installer).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn staged(version: &str, channel: &str) -> Staged {
        Staged {
            version: version.into(),
            channel: channel.into(),
            manifest: serde_json::json!({"version": version}),
        }
    }
    #[test]
    fn a_staged_update_round_trips_and_half_written_ones_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("updates").join("staged");
        assert!(read(&path).is_none());
        write(&path, &staged("1.2.0", "stable"), b"installer").unwrap();
        let (meta, bytes) = read(&path).unwrap();
        assert_eq!(
            (meta, bytes),
            (staged("1.2.0", "stable"), b"installer".to_vec())
        );
        write(&path, &staged("1.3.0", "stable"), b"newer").unwrap();
        assert_eq!(
            read(&path).unwrap().0.version,
            "1.3.0",
            "a newer download replaces the older one"
        );
        std::fs::remove_file(path.join(META)).unwrap();
        assert!(read(&path).is_none(), "no metadata means not staged");
        write(&path, &staged("1.3.0", "stable"), b"x").unwrap();
        std::fs::write(path.join(META), b"not json").unwrap();
        assert!(read(&path).is_none());
        write(&path, &staged("1.3.0", "stable"), b"x").unwrap();
        std::fs::write(path.join(INSTALLER), b"").unwrap();
        assert!(read(&path).is_none(), "an empty installer is never used");
        clear(&path);
        assert!(!path.exists());
    }
    #[test]
    fn only_a_newer_update_of_the_same_channel_with_auto_update_on_is_installable() {
        let update = staged("1.2.0", "stable");
        assert!(installable(&update, "1.1.0", "stable", true));
        assert!(
            !installable(&update, "1.2.0", "stable", true),
            "already running it"
        );
        assert!(
            !installable(&update, "1.3.0", "stable", true),
            "never a downgrade"
        );
        assert!(
            !installable(&update, "1.1.0", "beta", true),
            "another channel"
        );
        assert!(
            !installable(&update, "1.1.0", "stable", false),
            "auto update turned off"
        );
        assert!(!installable(
            &staged("not-semver", "stable"),
            "1.1.0",
            "stable",
            true
        ));
        assert!(
            installable(&staged("1.2.0", "stable"), "1.2.0-beta.4", "stable", true),
            "a beta is older than its release"
        );
    }
}
