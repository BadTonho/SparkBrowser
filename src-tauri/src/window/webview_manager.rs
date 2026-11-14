use super::internal_overlays::{
    spawn_bookmark_context_menu_webview, spawn_workspace_switcher_webview,
};
use crate::privacy::adblock_engine::should_block_url;
use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::state::tab_model::webview_label_for_tab;
use tauri::{AppHandle, Manager, Webview};

pub use super::content_focus::focus_active_content_webview;
pub use super::content_lifecycle::{
    close_tab_webview, navigate_tab_document, prepare_content_reset, reset_content_webviews,
    sync_all_webviews,
};
pub use super::internal_overlays::{
    build_bookmark_context_menu_script, build_workspace_switcher_update_script,
    create_internal_bridge_token, BookmarkContextMenuAction, OverlayLanguage, OverlayPresentation,
};
pub use super::url_input::{resolve_input_to_url_with_search_engine, search_engine_home_url};
pub const CONTENT_WEBVIEW_PREFIX: &str = "content-webview-";
pub const WORKSPACE_SWITCHER_WEBVIEW_LABEL: &str = "workspace-switcher-webview";
pub const BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL: &str = "bookmark-context-menu-webview";

/// The page the main window shows. Code that has no window to work with yet uses this.
pub fn active_content_webview(app: &AppHandle) -> Option<Webview> {
    active_content_webview_in(app, crate::state::tab_model::MAIN_WINDOW_ID)
}

/// The page one window shows.
pub fn active_content_webview_in(app: &AppHandle, window_id: &str) -> Option<Webview> {
    let state = app.try_state::<BrowserState>()?;
    let id = state
        .session()
        .window(window_id)?
        .active_tab_id()
        .lock()
        .ok()?
        .clone();
    app.get_webview(&webview_label_for_tab(&id))
}

pub fn ensure_active_content_webview(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<BrowserState>();
    let id = state
        .session()
        .active_tab_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let tab = state
        .session()
        .tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|tab| tab.id == id)
        .cloned()
        .ok_or("Active tab was not found")?;
    let bounds = *state
        .surfaces()
        .bounds()
        .lock()
        .map_err(|e| e.to_string())?;
    ensure_tab_webview(app, &tab.id, &tab.url, bounds)
}

pub fn spawn_content_webview(app: &AppHandle) -> Result<(), String> {
    ensure_active_content_webview(app)?;
    let bounds = *app
        .state::<BrowserState>()
        .surfaces()
        .bounds()
        .lock()
        .map_err(|e| e.to_string())?;
    spawn_workspace_switcher_webview(app, bounds)?;
    spawn_bookmark_context_menu_webview(app)?;
    sync_all_webviews(app, &app.state::<BrowserState>())?;
    Ok(())
}

pub(super) fn ensure_tab_webview(
    app: &AppHandle,
    tab_id: &str,
    url: &str,
    bounds: ViewportBounds,
) -> Result<(), String> {
    super::content_lifecycle::ensure_tab_webview(app, tab_id, url, bounds)
}

pub fn navigate_content_webview(app: &AppHandle, target_url_str: &str) -> Result<String, String> {
    navigate_content_webview_with_search_engine(app, target_url_str, "google")
}

/// Navigates the persistent child WebView using a supported search engine for text queries.
pub fn navigate_content_webview_with_search_engine(
    app: &AppHandle,
    target_url_str: &str,
    search_engine: &str,
) -> Result<String, String> {
    navigate_window_with_search_engine(
        app,
        crate::state::tab_model::MAIN_WINDOW_ID,
        target_url_str,
        search_engine,
    )
}

/// Navigates the page one window shows.
pub fn navigate_window_with_search_engine(
    app: &AppHandle,
    window_id: &str,
    target_url_str: &str,
    search_engine: &str,
) -> Result<String, String> {
    let parsed_url = super::url_input::resolve_browser_input(app, target_url_str, search_engine)?;
    let resolved_str = parsed_url.to_string();
    let state = app.state::<BrowserState>();
    let enabled = *state
        .policies()
        .shield_enabled()
        .lock()
        .map_err(|e| e.to_string())?;
    if crate::privacy::adblock_engine::site_is_protected(&state, &resolved_str)
        && should_block_url(&resolved_str, enabled)
    {
        return Err("Navigation blocked by the privacy shield".to_string());
    }

    let active = state
        .session()
        .window(window_id)
        .ok_or("windows.not_found")?
        .active_tab_id()
        .lock()?
        .clone();
    navigate_tab_document(
        app,
        &active,
        super::new_tab::physical_url(app, parsed_url.as_str())?,
    )
    .map_err(|e| format!("Navigation error: {e}"))?;

    Ok(resolved_str)
}

pub fn go_back_content_webview(app: &AppHandle, window_id: &str) -> Result<(), String> {
    navigate_content_history(app, window_id, true)
}

pub fn go_forward_content_webview(app: &AppHandle, window_id: &str) -> Result<(), String> {
    navigate_content_history(app, window_id, false)
}

fn navigate_content_history(app: &AppHandle, window_id: &str, go_back: bool) -> Result<(), String> {
    let webview = active_content_webview_in(app, window_id)
        .ok_or_else(|| "Content WebView was not found".to_string())?;

    #[cfg(windows)]
    {
        use tauri::webview::PlatformWebview;

        let operation = webview.with_webview(move |platform: PlatformWebview| {
            let result = unsafe {
                platform.controller().CoreWebView2().and_then(|core| {
                    if go_back {
                        core.GoBack()
                    } else {
                        core.GoForward()
                    }
                })
            };
            if let Err(error) = result {
                eprintln!("Failed to navigate content history: {error}");
            }
        });
        operation.map_err(|error| format!("Failed to access content WebView: {error}"))?;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let script = if go_back {
            "window.history.back();"
        } else {
            "window.history.forward();"
        };
        webview.eval(script).map_err(|error| error.to_string())
    }
}
