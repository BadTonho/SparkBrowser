//! Browser windows beyond the first one. Each has its own shell, tabs and page area; the profile,
//! workspaces and settings are shared.
use crate::state::{
    browser_state::BrowserState,
    session_ops::windows::{close_window, next_window_id, open_window},
    tab_model::MAIN_WINDOW_ID,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// A window the user can send a tab to.
#[derive(Clone, Serialize)]
pub struct WindowSummary {
    pub id: String,
    pub tab_count: usize,
}

fn build(app: &AppHandle, id: &str) -> Result<WebviewWindow, String> {
    #[allow(unused_mut)]
    let mut builder = WebviewWindowBuilder::new(app, id, WebviewUrl::App("index.html".into()))
        .title("Spark")
        .inner_size(1280.0, 820.0)
        .min_inner_size(680.0, 480.0)
        .decorations(false)
        .transparent(true)
        .center();
    #[cfg(windows)]
    {
        builder = builder.browser_extensions_enabled(true);
    }
    let window = builder.build().map_err(|e| e.to_string())?;
    #[cfg(target_os = "linux")]
    {
        let shell = window.clone();
        app.run_on_main_thread(move || {
            let _ = super::linux_layout::setup(&shell);
        })
        .map_err(|e| e.to_string())?;
    }
    super::vibrancy::apply_native_vibrancy(&window);
    if let Some(view) = app.get_webview(id) {
        crate::security::harden(app, &view, true)?;
        crate::webview_runtime::initialize(app, &view)?;
    }
    Ok(window)
}

/// Opens a new window showing a new-tab page in the workspace `from_window` has selected.
pub async fn open(app: &AppHandle, from_window: &str) -> Result<String, String> {
    let state = app.state::<BrowserState>();
    let _operation = state.session().operation().await;
    let workspace = state
        .session()
        .window(from_window)
        .and_then(|window| {
            window
                .active_workspace_id()
                .lock()
                .ok()
                .map(|id| id.clone())
        })
        .unwrap_or_else(|| crate::state::workspace_model::DEFAULT_WORKSPACE_ID.to_string());
    let id = next_window_id(&state);
    open_window(&state, &id, &workspace)?;
    if let Err(error) = build(app, &id) {
        // The window never appeared: take it out of the session again, keeping its tab as closed.
        crate::error_log::ignored(close_window(&state, &id));
        return Err(error);
    }
    super::webview_manager::sync_all_webviews(app, &state)?;
    crate::state::publish::publish_tabs(app, &state, from_window);
    Ok(id)
}

/// Sends a tab to another window. The page is rebuilt inside that window, so it reloads.
pub async fn move_tab(app: &AppHandle, tab_id: &str, target: &str) -> Result<(), String> {
    let state = app.state::<BrowserState>();
    let _operation = state.session().operation().await;
    let source = crate::state::session_ops::windows::move_tab(&state, tab_id, target)?;
    if source == target {
        return Ok(());
    }
    super::webview_manager::close_tab_webview(app, tab_id)?;
    super::webview_manager::sync_all_webviews(app, &state)?;
    for id in [source.as_str(), target] {
        crate::state::publish::publish_tabs(app, &state, id);
    }
    Ok(())
}

/// Closes a window the user closed: its tabs move to the recently closed list.
pub async fn closed(app: &AppHandle, window_id: &str) -> Result<(), String> {
    let state = app.state::<BrowserState>();
    let _operation = state.session().operation().await;
    super::overlay_host::release_window(app, window_id);
    let closed = close_window(&state, window_id)?;
    for tab in &closed.tab_ids {
        if let Err(error) = super::webview_manager::close_tab_webview(app, tab) {
            crate::error_log::ignored(crate::security::emit_shell(app, "browser-error", error));
        } else if closed.ephemeral_tab_ids.contains(tab) {
            super::ephemeral::purge_ephemeral_tab_data(app, tab);
        }
    }
    #[cfg(target_os = "linux")]
    super::linux_layout::forget_window(window_id);
    state.surfaces().forget_window_geometry(window_id);
    if let Some(window) = app.get_webview_window(window_id) {
        crate::error_log::ignored(window.destroy());
    }
    crate::state::session_ops::closed_tabs::notify(app, &state);
    crate::state::publish::publish_tabs(app, &state, MAIN_WINDOW_ID);
    Ok(())
}

pub fn summaries(state: &BrowserState) -> Vec<WindowSummary> {
    let tabs = state.snapshot_tabs();
    state
        .session()
        .window_ids()
        .into_iter()
        .map(|id| WindowSummary {
            tab_count: tabs.iter().filter(|tab| tab.window_id == id).count(),
            id,
        })
        .collect()
}

/// Remembers where a window sits so the next launch can put it back. A minimized window keeps its
/// previous place, and a maximized one keeps the size it had before.
pub fn capture_geometry(window: &tauri::Window) {
    if !crate::state::tab_model::is_window_label(window.label()) {
        return;
    }
    let app = window.app_handle();
    let Some(state) = app.try_state::<BrowserState>() else {
        return;
    };
    if window.is_minimized().unwrap_or(false) || state.session().window(window.label()).is_none() {
        return;
    }
    let mut record = state
        .surfaces()
        .window_geometry(window.label())
        .unwrap_or_default();
    let maximized = window.is_maximized().unwrap_or(false);
    record.maximized = maximized;
    if !maximized && !window.is_fullscreen().unwrap_or(false) {
        if let (Ok(position), Ok(size)) = (window.outer_position(), window.inner_size()) {
            record.x = Some(position.x);
            record.y = Some(position.y);
            record.width = Some(size.width);
            record.height = Some(size.height);
        }
    }
    record.monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .and_then(|monitor| monitor.name().cloned());
    state.surfaces().set_window_geometry(window.label(), record);
}

/// Refreshes the remembered place of every window, so the final save is exact.
pub fn capture_all(app: &AppHandle) {
    for (label, window) in app.windows() {
        if crate::state::tab_model::is_window_label(&label) {
            capture_geometry(&window);
        }
    }
}

/// True where an application may choose where its windows go. Wayland compositors decide that
/// themselves, so there only the size and the maximized state come back.
fn can_place_windows() -> bool {
    #[cfg(windows)]
    {
        true
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("WAYLAND_DISPLAY").is_none()
            || std::env::var("GDK_BACKEND").is_ok_and(|value| value.contains("x11"))
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        false
    }
}

/// Puts a window back where it was. A place that no monitor shows any more is ignored, so the
/// window never reopens out of reach.
pub fn apply_geometry(
    window: &WebviewWindow,
    record: &crate::database::session_windows_repo::WindowRecord,
) {
    use tauri::{PhysicalPosition, PhysicalSize};
    if let (Some(width), Some(height)) = (record.width, record.height) {
        if width >= 400 && height >= 300 {
            crate::error_log::ignored(window.set_size(PhysicalSize::new(width, height)));
        }
    }
    if can_place_windows() {
        if let (Some(x), Some(y)) = (record.x, record.y) {
            let visible = window.available_monitors().is_ok_and(|monitors| {
                monitors.iter().any(|monitor| {
                    let (origin, size) = (monitor.position(), monitor.size());
                    x >= origin.x - 50
                        && y >= origin.y - 50
                        && x < origin.x + size.width as i32 - 100
                        && y < origin.y + size.height as i32 - 100
                })
            });
            if visible {
                crate::error_log::ignored(window.set_position(PhysicalPosition::new(x, y)));
            }
        }
    }
    if record.maximized {
        crate::error_log::ignored(window.maximize());
    }
}

/// Opens the windows the last session had, each with its own tabs, and puts every window back where
/// it was. A window that cannot open keeps its tabs: they go to the recently closed list.
pub async fn restore_saved(app: &AppHandle) {
    let state = app.state::<BrowserState>();
    let _operation = state.session().operation().await;
    let ids = state.session().window_ids();
    for id in ids {
        let record = state.surfaces().window_geometry(&id);
        if id == MAIN_WINDOW_ID {
            if let (Some(window), Some(record)) = (app.get_webview_window(&id), record) {
                apply_geometry(&window, &record);
            }
            continue;
        }
        match build(app, &id) {
            Ok(window) => {
                if let Some(record) = record {
                    apply_geometry(&window, &record);
                }
            }
            Err(error) => {
                eprintln!("Could not reopen {id}: {error}");
                if let Ok(closed) = close_window(&state, &id) {
                    for tab in &closed.tab_ids {
                        crate::error_log::ignored(super::webview_manager::close_tab_webview(
                            app, tab,
                        ));
                    }
                }
                crate::error_log::ignored(crate::security::emit_shell(app, "browser-error", error));
            }
        }
    }
    if let Err(error) = super::webview_manager::sync_all_webviews(app, &state) {
        crate::error_log::ignored(crate::security::emit_shell(app, "browser-error", error));
    }
    crate::state::session_ops::closed_tabs::notify(app, &state);
    for id in state.session().window_ids() {
        crate::state::publish::publish_tabs(app, &state, &id);
    }
}
