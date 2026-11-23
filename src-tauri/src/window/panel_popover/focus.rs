use tauri::{AppHandle, Manager};

pub(super) fn lost<P: super::Panel>(app: &AppHandle) {
    let Some(window) = crate::window::overlay_host::window_of(app, P::LABEL) else {
        return;
    };
    let Some(session_id) = app
        .state::<super::Popover<P>>()
        .0
        .lock()
        .ok()
        .and_then(|inner| inner.session.as_ref().map(|session| session.id))
    else {
        return;
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        // Let focus settle when moving between the shell and an auxiliary child.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        if !focused(&window) {
            super::dismiss_session::<P>(&app, Some(session_id));
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
    // WebView2 can report the shell as blurred when a child or one of its owned
    // windows takes focus. GA_ROOT and GA_ROOTOWNER keep that focus in the shell.
    unsafe {
        let foreground = GetForegroundWindow();
        GetAncestor(foreground, 2) == handle.0 || GetAncestor(foreground, 3) == handle.0
    }
}

#[cfg(not(windows))]
pub(super) fn focused(window: &tauri::Window) -> bool {
    window.is_focused().unwrap_or(false)
}
