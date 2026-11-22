mod http;
pub mod model;
#[cfg(any(windows, test))]
mod releases;
mod service;
#[cfg_attr(not(windows), allow(dead_code))] // the background installer is Windows only
pub(crate) mod staged;
#[cfg(windows)]
mod windows;
pub use service::UpdateService;
#[cfg(windows)]
pub use windows::{download, install};

/// Updates only exist for the installed Windows x64 release, and only the Default profile installs them:
/// two profiles open at once must not run the same installer.
pub(crate) fn platform_supported(app: &tauri::AppHandle) -> bool {
    cfg!(all(windows, target_arch = "x86_64", not(debug_assertions)))
        && app.config().identifier == "org.spark.browser"
        && !is_portable()
        && crate::profiles::is_default_profile(app)
}

/// The portable zip ships a `portable.txt` next to the executable. Updates run the installer, which
/// would turn a portable copy into an installed one, so a portable copy does not update itself.
pub(crate) fn is_portable() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("portable.txt")))
        .is_some_and(|marker| marker.is_file())
}

/// Installs an update that was downloaded in the background, before the previous session is restored.
/// Returns when there is nothing to install (or it could not be installed); on success the installer
/// takes over and the process exits.
pub fn install_staged_at_startup(app: &tauri::AppHandle, conn: &rusqlite::Connection) {
    #[cfg(windows)]
    {
        use tauri::Manager;
        if !platform_supported(app)
            || app
                .try_state::<crate::private_session::PrivateRole>()
                .is_some_and(|role| role.is_private())
        {
            return;
        }
        let version = app.package_info().version.to_string();
        let Ok((channel, _)) =
            crate::database::updates_repo::load(conn, model::UpdateChannel::initial(&version))
        else {
            return;
        };
        let auto_update = crate::database::updates_repo::load_auto_update(conn).unwrap_or(true);
        if let Err(error) = staged::install_pending(app, &version, channel, auto_update) {
            crate::error_log::warn(&format!("Staged update was not installed: {error}"));
        }
    }
    #[cfg(not(windows))]
    let _ = (app, conn);
}

pub fn initialize_crypto() {
    #[cfg(windows)]
    crate::error_log::ignored(rustls::crypto::ring::default_provider().install_default());
}

pub(crate) fn validate_restart(
    tabs: &[crate::state::tab_model::TabInfo],
    pending_downloads: bool,
    confirmed_activity: bool,
) -> Result<(), String> {
    if pending_downloads {
        return Err("Finish or cancel active downloads before installing the update.".into());
    }
    if tabs.iter().any(|tab| tab.activity.downloading) {
        return Err("A page download is still active. Finish it before updating.".into());
    }
    if !confirmed_activity
        && tabs.iter().any(|tab| {
            tab.activity.dirty_form || tab.activity.playing_media || tab.activity.picture_in_picture
        })
    {
        return Err(
            "Confirm closing pages with edited forms or playing media before updating.".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn restart_revalidates_downloads_and_requires_explicit_activity_confirmation() {
        let mut tab = crate::state::tab_model::TabInfo::new_default(
            "test".into(),
            "https://example.test".into(),
            true,
        );
        assert!(super::validate_restart(&[tab.clone()], true, true).is_err());
        tab.activity.dirty_form = true;
        assert!(super::validate_restart(&[tab.clone()], false, false).is_err());
        assert!(super::validate_restart(&[tab.clone()], false, true).is_ok());
        tab.activity.dirty_form = false;
        tab.activity.playing_media = true;
        assert!(super::validate_restart(&[tab.clone()], false, false).is_err());
        tab.activity.downloading = true;
        assert!(super::validate_restart(&[tab], false, true).is_err());
    }
}
