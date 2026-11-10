#[cfg(target_os = "linux")]
mod caret_browsing;
pub(crate) mod content_bridge;
mod content_focus;
mod content_lifecycle;
pub mod favicon;
pub mod fullscreen;
pub mod history_overlay;
mod internal_overlays;
mod keyboard_bridge;
mod link_context_menu;
pub mod media_popover;
pub(crate) mod navigation_state;
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

#[cfg(windows)]
pub fn bring_webview_to_front(view: &tauri::Webview) {
    let _ = view.with_webview(move |platform| {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn SetWindowPos(
                hwnd: *mut std::ffi::c_void,
                hwnd_insert_after: *mut std::ffi::c_void,
                x: i32,
                y: i32,
                cx: i32,
                cy: i32,
                flags: u32,
            ) -> i32;
        }
        const SWP_NOSIZE: u32 = 0x0001;
        const SWP_NOMOVE: u32 = 0x0002;
        const SWP_NOACTIVATE: u32 = 0x0010;
        unsafe {
            let mut parent = Default::default();
            if platform.controller().ParentWindow(&mut parent).is_ok() && !parent.0.is_null() {
                SetWindowPos(
                    parent.0,
                    std::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    });
}

#[cfg(not(windows))]
pub fn bring_webview_to_front(_view: &tauri::Webview) {}

#[cfg(windows)]
pub fn send_webview_to_back(view: &tauri::Webview) {
    let _ = view.with_webview(move |platform| {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn SetWindowPos(
                hwnd: *mut std::ffi::c_void,
                hwnd_insert_after: *mut std::ffi::c_void,
                x: i32,
                y: i32,
                cx: i32,
                cy: i32,
                flags: u32,
            ) -> i32;
        }
        const SWP_NOSIZE: u32 = 0x0001;
        const SWP_NOMOVE: u32 = 0x0002;
        const SWP_NOACTIVATE: u32 = 0x0010;
        unsafe {
            let mut parent = Default::default();
            if platform.controller().ParentWindow(&mut parent).is_ok() && !parent.0.is_null() {
                SetWindowPos(
                    parent.0,
                    std::ptr::dangling_mut::<std::ffi::c_void>(),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    });
}

#[cfg(not(windows))]
pub fn send_webview_to_back(_view: &tauri::Webview) {}

pub fn show_overlay_webview(view: &tauri::Webview) -> Result<(), String> {
    view.show().map_err(|e| e.to_string())?;
    bring_webview_to_front(view);
    Ok(())
}

mod content_factory;
