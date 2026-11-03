use crate::state::{
    browser_state::BrowserState,
    split_docking::{self, DockContext, DockSide},
    split_layout::pane_bounds,
};
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{
    webview::WebviewBuilder, AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition,
    Rect, WebviewUrl, WindowBuilder,
};

const LABEL: &str = "tab-drag-preview";
#[derive(Default)]
pub struct DragPreview(pub Mutex<u64>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    tab_id: String,
    side: DockSide,
    context: DockContext,
    label: String,
    locale: String,
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_window(LABEL) {
        let _ = visibility(&window, false);
    }
}

/// Tao's later show() may activate even a non-focusable owned window on Windows.
/// Keep the real pointer gesture in the shell by showing without activation.
#[cfg(windows)]
fn visibility(window: &tauri::Window, visible: bool) -> Result<(), String> {
    let handle = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    window
        .run_on_main_thread(move || {
            #[link(name = "user32")]
            unsafe extern "system" {
                fn ShowWindow(window: *mut std::ffi::c_void, command: i32) -> i32;
            }
            unsafe {
                ShowWindow(handle as *mut std::ffi::c_void, if visible { 4 } else { 0 });
            }
        })
        .map_err(|e| e.to_string())
}
#[cfg(not(windows))]
fn visibility(window: &tauri::Window, visible: bool) -> Result<(), String> {
    if visible {
        window.show()
    } else {
        window.hide()
    }
    .map_err(|e| e.to_string())
}

pub fn blocked(state: &BrowserState) -> bool {
    state.is_overlay_open.lock().map_or(true, |value| *value)
        || state
            .workspace_switcher_open
            .lock()
            .map_or(true, |value| *value)
        || state
            .bookmark_context_menu_open
            .lock()
            .map_or(true, |value| *value)
}

#[tauri::command]
pub async fn set_tab_drag_preview(
    app: AppHandle,
    state: tauri::State<'_, BrowserState>,
    revision: u64,
    presentation: Option<Presentation>,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    let service = app.state::<DragPreview>();
    {
        let mut latest = service.0.lock().map_err(|e| e.to_string())?;
        if revision <= *latest {
            return Ok(());
        }
        *latest = revision;
    }
    let Some(presentation) = presentation else {
        hide(&app);
        return Ok(());
    };
    if blocked(&state) || app.state::<crate::shutdown::ShutdownService>().blocked() {
        hide(&app);
        return Err("A browser panel is open".into());
    }
    let next = split_docking::plan(
        &state,
        &presentation.context,
        &presentation.tab_id,
        presentation.side,
    )
    .inspect_err(|_| hide(&app))?;
    let bounds = *state.bounds.lock().map_err(|e| e.to_string())?;
    let pane = pane_bounds(bounds, &next)[if presentation.side == DockSide::Left {
        0
    } else {
        1
    }]
    .1;
    let main = app.get_window("main").ok_or("Main window not found")?;
    if main.is_minimized().map_err(|e| e.to_string())? {
        hide(&app);
        return Ok(());
    }
    let scale = main.scale_factor().map_err(|e| e.to_string())?;
    let origin = main.inner_position().map_err(|e| e.to_string())?;
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
        let window = WindowBuilder::new(&app, LABEL)
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
            .inner_size(pane.width, pane.height)
            .build()
            .map_err(|e| e.to_string())?;
        window
            .add_child(
                WebviewBuilder::new(LABEL, WebviewUrl::App("tab-drag-preview.html".into()))
                    .transparent(true)
                    .focused(false)
                    .on_navigation(move |url| url == &document),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(pane.width, pane.height),
            )
            .map_err(|e| e.to_string())?;
        window
    };
    window
        .set_ignore_cursor_events(true)
        .map_err(|e| e.to_string())?;
    window
        .set_position(PhysicalPosition::new(
            (origin.x as f64 + pane.x * scale).round() as i32,
            (origin.y as f64 + pane.y * scale).round() as i32,
        ))
        .map_err(|e| e.to_string())?;
    window
        .set_size(LogicalSize::new(pane.width, pane.height))
        .map_err(|e| e.to_string())?;
    let payload = serde_json::json!({ "revision": revision, "label": presentation.label, "locale": presentation.locale });
    let view = app
        .get_webview(LABEL)
        .ok_or("Split preview WebView not found")?;
    view.set_bounds(Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(pane.width, pane.height).into(),
    })
    .map_err(|e| e.to_string())?;
    view.eval(format!("(() => {{ const data={payload}; let attempts=0; const paint=()=>{{if(window.setDragPreview)window.setDragPreview(data);else if(++attempts<120)setTimeout(paint,16);}};paint(); }})();")).map_err(|e| e.to_string())?;
    visibility(&window, true)
}
