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
        let _ = close_window(&state, &id);
        return Err(error);
    }
    super::webview_manager::sync_all_webviews(app, &state)?;
    crate::state::publish::publish_tabs(app, &state, from_window);
    Ok(id)
}

/// Closes a window the user closed: its tabs move to the recently closed list.
pub async fn closed(app: &AppHandle, window_id: &str) -> Result<(), String> {
    let state = app.state::<BrowserState>();
    let _operation = state.session().operation().await;
    let closed = close_window(&state, window_id)?;
    for tab in &closed.tab_ids {
        if let Err(error) = super::webview_manager::close_tab_webview(app, tab) {
            let _ = crate::security::emit_shell(app, "browser-error", error);
        } else if closed.ephemeral_tab_ids.contains(tab) {
            super::ephemeral::purge_ephemeral_tab_data(app, tab);
        }
    }
    #[cfg(target_os = "linux")]
    super::linux_layout::forget_window(window_id);
    if let Some(window) = app.get_webview_window(window_id) {
        let _ = window.destroy();
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
