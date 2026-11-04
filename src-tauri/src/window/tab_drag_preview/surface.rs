use tauri::{
    webview::WebviewBuilder, AppHandle, LogicalPosition, LogicalSize, Manager, Rect, WebviewUrl,
    WindowBuilder,
};

const LABEL: &str = "tab-drag-preview";

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_window(LABEL) {
        let _ = visibility(&window, None);
    }
}

// Showing an owned preview must preserve the shell's real pointer capture.
#[cfg(windows)]
fn visibility(window: &tauri::Window, revision: Option<u64>) -> Result<(), String> {
    let handle = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    let app = window.app_handle().clone();
    window
        .run_on_main_thread(move || {
            if revision.is_some_and(|revision| !super::paint_is_current(&app, revision)) {
                return;
            }
            #[link(name = "user32")]
            unsafe extern "system" {
                fn ShowWindow(window: *mut std::ffi::c_void, command: i32) -> i32;
            }
            unsafe {
                ShowWindow(
                    handle as *mut std::ffi::c_void,
                    if revision.is_some() { 4 } else { 0 },
                );
            }
        })
        .map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn visibility(window: &tauri::Window, revision: Option<u64>) -> Result<(), String> {
    let target = window.clone();
    window
        .run_on_main_thread(move || {
            if revision
                .is_some_and(|revision| !super::paint_is_current(target.app_handle(), revision))
            {
                return;
            }
            if revision.is_some() {
                let _ = target.show();
            } else {
                let _ = target.hide();
            }
        })
        .map_err(|e| e.to_string())
}

pub fn paint(app: &AppHandle, payload: &serde_json::Value) -> Result<(), String> {
    let revision = payload["revision"]
        .as_u64()
        .ok_or("Missing preview paint revision")?;
    if !super::paint_is_current(app, revision) {
        return Ok(());
    }
    let main = app.get_window("main").ok_or("Main window not found")?;
    if main.is_minimized().map_err(|e| e.to_string())? {
        hide(app);
        return Ok(());
    }
    let scale = main.scale_factor().map_err(|e| e.to_string())?;
    let origin = main.inner_position().map_err(|e| e.to_string())?;
    let size = main.inner_size().map_err(|e| e.to_string())?;
    let logical = size.to_logical::<f64>(scale);
    let window = if let Some(window) = app.get_window(LABEL) {
        window
    } else {
        let document = app
            .get_webview("main")
            .ok_or("Browser shell not found")?
            .url()
            .map_err(|e| e.to_string())?
            .join("/tab-drag-preview.html")
            .map_err(|e| e.to_string())?;
        let window = WindowBuilder::new(app, LABEL)
            .title("Veloce split preview")
            .parent(&main)
            .map_err(|e| e.to_string())?
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .shadow(false)
            .skip_taskbar(true)
            .focused(false)
            .focusable(false)
            .visible(false)
            .inner_size(logical.width, logical.height)
            .build()
            .map_err(|e| e.to_string())?;
        if let Err(error) = window.add_child(
            WebviewBuilder::new(LABEL, WebviewUrl::App("tab-drag-preview.html".into()))
                .transparent(true)
                .focused(false)
                .on_navigation(move |url| {
                    crate::security::documents::same_document(url, &document)
                }),
            LogicalPosition::new(0.0, 0.0),
            logical,
        ) {
            let _ = window.destroy();
            return Err(error.to_string());
        }
        if let Some(view) = app.get_webview(LABEL) {
            crate::security::harden(app, &view, true)?;
        }
        if let Err(error) = window.set_ignore_cursor_events(true) {
            let _ = window.destroy();
            return Err(error.to_string());
        }
        window
    };
    if window.outer_position().map_err(|e| e.to_string())? != origin {
        window.set_position(origin).map_err(|e| e.to_string())?;
    }
    if window.inner_size().map_err(|e| e.to_string())? != size {
        window.set_size(size).map_err(|e| e.to_string())?;
    }
    let view = app
        .get_webview(LABEL)
        .ok_or("Split preview WebView not found")?;
    let bounds = Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(logical.width, logical.height).into(),
    };
    if view.size().map_err(|e| e.to_string())? != size {
        view.set_bounds(bounds).map_err(|e| e.to_string())?;
    }
    view.eval(format!("(() => {{ const data={payload}; let attempts=0; const paint=()=>{{if(window.setDragPreview)window.setDragPreview(data);else if(++attempts<120)setTimeout(paint,16);}};paint(); }})();")).map_err(|e| e.to_string())?;
    visibility(&window, Some(revision))
}
