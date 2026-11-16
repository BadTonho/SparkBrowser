#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::install;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::install;

#[cfg(any(windows, target_os = "linux"))]
use crate::{database::site_icons_repo, state::browser_state::BrowserState};
use tauri::{AppHandle, Manager};

pub fn epoch(app: &AppHandle) -> u64 {
    app.state::<super::new_tab::NewTabService>()
        .history_epoch
        .load(std::sync::atomic::Ordering::SeqCst)
}

// Called under the database lock, so clearing history and finishing old captures are ordered.
#[cfg(any(windows, target_os = "linux"))]
pub fn save(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    expected_epoch: u64,
    key: &str,
    bytes: &[u8],
) -> Result<bool, String> {
    site_icons_repo::store_current(conn, key, bytes, expected_epoch, epoch(app))
}

#[cfg(any(windows, target_os = "linux"))]
fn current(app: &AppHandle, label: &str, token: &str, url: &str) -> bool {
    super::navigation_state::is_current_generation(app, label, token)
        && app
            .state::<BrowserState>()
            .snapshot_tabs()
            .iter()
            .any(|tab| {
                crate::state::tab_model::webview_label_for_tab(&tab.id) == label && tab.url == url
            })
}
