use tauri::AppHandle;

pub(super) fn lost<P: super::Panel>(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        // Let focus settle when moving between the shell and an auxiliary child.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        if crate::window::overlay_host::window_of(&app, <P as super::Panel>::LABEL)
            .is_none_or(|window| !focused(&window))
        {
            super::dismiss::<P>(&app);
        }
    });
}

#[cfg(windows)]
pub(super) fn focused(window: &tauri::Window) -> bool {
    use std::ffi::c_void;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetForegroundWindow() -> *mut c_void;
        fn GetAncestor(window: *mut c_void, flags: u32) -> *mut c_void;
    }
    let Ok(handle) = window.hwnd() else {
        return false;
    };
    // A focused WebView2 child can report the parent as blurred; its native root
    // still identifies the active browser window. GA_ROOT = 2.
    unsafe { GetAncestor(GetForegroundWindow(), 2) == handle.0 }
}

#[cfg(not(windows))]
pub(super) fn focused(window: &tauri::Window) -> bool {
    window.is_focused().unwrap_or(false)
}
