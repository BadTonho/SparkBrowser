//! Memory level of the tabs the user is not looking at.
//!
//! When a memory limit is on, the tabs that cannot be hibernated (a call, music, a form being edited, pinned
//! tabs) are still asked to use as little memory as they can while they are in the background. On Windows this
//! is the engine's own "memory usage target level" (`Low` trims what a hidden page can give back; `Normal` is
//! the default). The engine has no such control on Linux, so there it does nothing and hibernation is the only
//! lever.
use tauri::{AppHandle, Manager};

/// Asks the engine to keep the tab's page at the low (`true`) or the normal (`false`) memory level.
/// A tab without a page (a hibernated one) has nothing to change.
pub fn set_tab_level(app: &AppHandle, tab_id: &str, low: bool) -> Result<(), String> {
    let label = crate::state::tab_model::webview_label_for_tab(tab_id);
    let Some(view) = app.get_webview(&label) else {
        return Ok(());
    };
    #[cfg(windows)]
    {
        let tab_id = tab_id.to_string();
        view.with_webview(move |platform| {
            use webview2_com::Microsoft::Web::WebView2::Win32::{
                ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
            };
            use windows_core::Interface;
            let level = if low {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
            } else {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
            };
            let result = unsafe { platform.controller().CoreWebView2() }
                .and_then(|core| core.cast::<ICoreWebView2_19>())
                .and_then(|core| unsafe { core.SetMemoryUsageTargetLevel(level) });
            if let Err(error) = result {
                crate::error_log::warn(&format!(
                    "Could not set the memory level of tab {tab_id} (memory limit): {error}"
                ));
            }
        })
        .map_err(|error| error.to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = (view, low);
        Ok(())
    }
}
