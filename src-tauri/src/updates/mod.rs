mod http;
pub mod model;
#[cfg(any(windows, test))]
mod releases;
mod service;
#[cfg(windows)]
mod windows;
pub use service::UpdateService;
#[cfg(windows)]
pub use windows::{download, install};

pub fn initialize_crypto() {
    #[cfg(windows)]
    let _ = rustls::crypto::ring::default_provider().install_default();
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
