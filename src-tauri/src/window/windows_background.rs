//! The color WebView2 shows before a page has painted. Without it an internal page of the app is
//! white for as long as it takes to load, which on a dark app looks like a broken page.
use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2Controller2, COREWEBVIEW2_COLOR};
use windows_core::Interface;

/// Matches `--bg-viewport-frame` of the dark theme.
const DARK: COREWEBVIEW2_COLOR = COREWEBVIEW2_COLOR {
    A: 255,
    R: 16,
    G: 16,
    B: 16,
};

pub fn dark_until_painted(view: &tauri::Webview) {
    crate::error_log::ignored(view.with_webview(|platform| unsafe {
        match platform.controller().cast::<ICoreWebView2Controller2>() {
            Ok(controller) => {
                crate::error_log::ignored(controller.SetDefaultBackgroundColor(DARK));
            }
            Err(error) => {
                crate::error_log::warn(&format!("Could not set the page background: {error}"));
            }
        }
    }));
}
