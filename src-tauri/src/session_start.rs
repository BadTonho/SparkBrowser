//! The last part of start-up: everything that brings the profile's pages onto the screen (the content
//! view, the windows of the last session, the filter-list updates, the feeds) and the moment the
//! browser becomes ready for commands.
//!
//! It is a function of its own because a profile with an app lock stops before it: nothing of the
//! profile is shown until the lock is opened, and opening it runs this.
use crate::state::browser_state::BrowserState;
use crate::window::webview_manager::spawn_content_webview;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};

pub fn start(app: &AppHandle) -> Result<(), String> {
    spawn_content_webview(app)?;
    if !cfg!(test) && app.config().identifier != "org.spark.browser.native-tests" {
        app.state::<std::sync::Arc<crate::privacy::filter_lists::FilterLists>>()
            .start_updates();
    }
    // The other windows of the last session come back once the first one is up.
    let reopening = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::window::browser_windows::restore_saved(&reopening).await;
    });
    app.state::<BrowserState>()
        .infrastructure()
        .startup_ready()
        .store(true, Ordering::Release);
    crate::feeds::start(app.clone());
    Ok(())
}
