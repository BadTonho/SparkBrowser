#[cfg(target_os = "linux")]
mod caret_browsing;
mod content_focus;
mod content_lifecycle;
pub mod favicon;
pub mod history_overlay;
mod internal_overlays;
mod keyboard_bridge;
pub mod media_popover;
mod navigation_state;
pub mod new_tab;
pub mod permission_popover;
pub mod privacy_popover;
pub mod reader;
pub mod shield_policy;
mod split_focus;
pub mod tab_drag_preview;
pub mod tab_memory;
pub(crate) mod url_input;
pub mod vibrancy;
pub mod viewport;
pub mod webview_manager;
pub mod workspace_data;
pub mod zoom_popover;

pub mod tools_popover;

pub mod downloads_popover;
pub mod ephemeral;
#[cfg(target_os = "linux")]
pub mod linux_layout;
mod panel_popover;
pub mod quick_settings_popover;
pub mod settings_overlay;

pub fn create_webview_builder(
    label: &str,
    url: tauri::WebviewUrl,
) -> tauri::webview::WebviewBuilder<tauri::Wry> {
    #[allow(unused_mut)]
    let mut builder = tauri::webview::WebviewBuilder::new(label, url);
    #[cfg(windows)]
    {
        builder = builder.browser_extensions_enabled(true);
    }
    builder
}
