use super::internal_overlays::{
    handle_workspace_switcher_navigation, spawn_bookmark_context_menu_webview,
    spawn_workspace_switcher_webview,
};
#[cfg(windows)]
use super::keyboard_bridge::install_workspace_shortcut_accelerator;
use super::keyboard_bridge::{build_workspace_shortcut_script, WORKSPACE_SWITCHER_ESCAPE_SCRIPT};
use super::navigation_state::update_navigation;
use crate::privacy::adblock_engine::{build_content_shield_script, should_block_url};
use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::state::tab_model::webview_label_for_tab;
use tauri::{
    webview::WebviewBuilder, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Webview,
    WebviewUrl,
};

pub use super::content_focus::focus_active_content_webview;
pub use super::content_lifecycle::{reset_content_webviews, sync_all_webviews};
pub use super::internal_overlays::{
    build_bookmark_context_menu_script, build_workspace_switcher_update_script,
    create_internal_bridge_token, BookmarkContextMenuAction, OverlayLanguage, OverlayPresentation,
};
pub use super::url_input::{resolve_input_to_url_with_search_engine, search_engine_home_url};
pub const CONTENT_WEBVIEW_PREFIX: &str = "content-webview-";
pub const WORKSPACE_SWITCHER_WEBVIEW_LABEL: &str = "workspace-switcher-webview";
pub const BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL: &str = "bookmark-context-menu-webview";

pub fn active_content_webview(app: &AppHandle) -> Option<Webview> {
    let state = app.try_state::<BrowserState>()?;
    let id = state.active_tab_id.lock().ok()?.clone();
    app.get_webview(&webview_label_for_tab(&id))
}

pub fn ensure_active_content_webview(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<BrowserState>();
    let id = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let tab = state
        .tabs
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|tab| tab.id == id)
        .cloned()
        .ok_or("Active tab was not found")?;
    let bounds = *state.bounds.lock().map_err(|e| e.to_string())?;
    ensure_tab_webview(app, &tab.id, &tab.url, bounds)
}

pub fn spawn_content_webview(app: &AppHandle) -> Result<(), String> {
    ensure_active_content_webview(app)?;
    let bounds = *app
        .state::<BrowserState>()
        .bounds
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
    initial_url_str: &str,
    bounds: ViewportBounds,
) -> Result<(), String> {
    let label = webview_label_for_tab(tab_id);
    if app.get_webview(&label).is_some() {
        return Ok(());
    }

    let main_window = app
        .get_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let initial_url = resolve_input_to_url_with_search_engine(initial_url_str, "google")?;
    let shield_enabled = app
        .try_state::<BrowserState>()
        .and_then(|s| s.shield_enabled.lock().ok().map(|v| *v))
        .unwrap_or(true);
    if should_block_url(initial_url.as_str(), shield_enabled) {
        return Err("Navigation blocked by the privacy shield".to_string());
    }
    let keyboard_bridge_token = create_internal_bridge_token();
    if let Some(state) = app.try_state::<BrowserState>() {
        if let Ok(mut tokens) = state.keyboard_bridge_tokens.lock() {
            tokens.insert(label.clone(), keyboard_bridge_token.clone());
        }
    }
    #[cfg(windows)]
    let native_alt_capture = true;
    #[cfg(not(windows))]
    let native_alt_capture = false;

    let app_handle_for_nav = app.clone();

    let focused = app
        .state::<BrowserState>()
        .active_tab_id
        .lock()
        .map(|id| *id == tab_id)
        .unwrap_or(false);
    #[cfg(windows)]
    let initial_target = initial_url.to_string();
    #[cfg(windows)]
    let initial_url = tauri::Url::parse("about:blank").expect("static blank URL");
    let webview_builder = WebviewBuilder::new(&label, WebviewUrl::External(initial_url))
        .initialization_script(
            include_str!("../media/controller.js")
                .replace(
                    "__MEDIA_TOKEN__",
                    &serde_json::to_string(&keyboard_bridge_token).expect("token serializable"),
                )
                .replace(
                    "__MEDIA_VOLUME__",
                    &app.state::<crate::media::MediaService>()
                        .tab(tab_id)
                        .volume
                        .to_string(),
                ),
        )
        .focused(focused)
        .zoom_hotkeys_enabled(false)
        .initialization_script(include_str!("page_activity.js").replace(
            "__ACTIVITY_TOKEN__",
            &serde_json::to_string(&keyboard_bridge_token).expect("token serializable"),
        ))
        .initialization_script(build_content_shield_script(
            shield_enabled,
            &keyboard_bridge_token,
        ))
        .initialization_script(WORKSPACE_SWITCHER_ESCAPE_SCRIPT);

    #[cfg(not(windows))]
    let webview_builder = webview_builder.on_download(crate::downloads::native::handler(
        app.clone(),
        tab_id.to_string(),
    ));

    let webview_builder = webview_builder.initialization_script(build_workspace_shortcut_script(
        &keyboard_bridge_token,
        native_alt_capture,
    ));

    let webview_builder = super::navigation_state::with_page_events(
        webview_builder,
        app,
        tab_id,
        &label,
        &keyboard_bridge_token,
    );

    #[cfg(windows)]
    let navigation_registration_token = keyboard_bridge_token.clone();
    let tab_for_nav = tab_id.to_string();
    let label_for_nav = label.clone();
    let webview_builder = webview_builder.on_navigation(move |nav_url| {
        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("page-activity") {
            let values: std::collections::HashMap<_, _> = nav_url.query_pairs().collect();
            if values.get("token").map(|v| v.as_ref()) == Some(keyboard_bridge_token.as_str()) {
                update_navigation(
                    &app_handle_for_nav,
                    &tab_for_nav,
                    &label_for_nav,
                    &keyboard_bridge_token,
                    |tab| {
                        tab.activity.dirty_form = values.get("dirty").is_some_and(|v| v == "true");
                        tab.activity.document_media =
                            values.get("media").is_some_and(|v| v == "true");
                        tab.activity.refresh_media();
                    },
                );
            }
            return false;
        }
        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("workspace-switcher")
        {
            handle_workspace_switcher_navigation(&app_handle_for_nav, nav_url);
            return false;
        }

        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("keyboard-shortcut")
        {
            super::keyboard_bridge::handle_keyboard_navigation(
                &app_handle_for_nav,
                nav_url,
                &tab_for_nav,
                &label_for_nav,
                &keyboard_bridge_token,
            );
            return false;
        }

        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("pane-focus") {
            if nav_url
                .query_pairs()
                .any(|(key, value)| key == "token" && value == keyboard_bridge_token)
            {
                super::split_focus::focus_pane(
                    &app_handle_for_nav,
                    &tab_for_nav,
                    &label_for_nav,
                    &keyboard_bridge_token,
                );
            }
            return false;
        }
        // No internal URL is a supported keyboard transport on Windows; keep
        // the channel scheme from ever reaching the web content as navigation.
        if nav_url.scheme() == "veloce-internal" {
            return false;
        }

        let workspace_picker_is_open = app_handle_for_nav
            .try_state::<BrowserState>()
            .and_then(|state| state.workspace_switcher_open.lock().ok().map(|open| *open))
            .unwrap_or(false);

        let is_active = app_handle_for_nav
            .try_state::<BrowserState>()
            .and_then(|state| state.active_tab_id.lock().ok().map(|id| *id == tab_for_nav))
            .unwrap_or(false);
        // Background pages must not cancel a picker held over the active tab.
        if is_active && !workspace_picker_is_open {
            if let Some(state) = app_handle_for_nav.try_state::<BrowserState>() {
                state
                    .workspace_alt_pressed
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                if let Ok(mut token) = state.workspace_switcher_token.lock() {
                    *token = None;
                }
                if let Ok(mut open) = state.workspace_switcher_open.lock() {
                    *open = false;
                }
            }
            let _ = app_handle_for_nav.emit("workspace-shortcut-cancel", ());
            let _ = app_handle_for_nav.emit("workspace-switcher-dismissed", ());
        }

        if !matches!(nav_url.scheme(), "http" | "https")
            || nav_url.host_str() == Some("tauri.localhost")
        {
            return false;
        }
        let new_url_str = nav_url.to_string();

        if let Some(state) = app_handle_for_nav.try_state::<BrowserState>() {
            let shield_on = state.shield_enabled.lock().map(|v| *v).unwrap_or(true);

            if should_block_url(&new_url_str, shield_on) {
                if let Ok(mut count) = state.blocked_count.lock() {
                    *count += 1;
                }
                let stats = state.snapshot_privacy_stats();
                let _ = app_handle_for_nav.emit("privacy-stats-updated", &stats);
                return false;
            }
        }
        true
    });

    main_window
        .add_child(
            webview_builder,
            LogicalPosition::new(bounds.x, bounds.y),
            LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)),
        )
        .map_err(|e| format!("Failed to create content WebView: {}", e))?;

    #[cfg(windows)]
    install_workspace_shortcut_accelerator(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
    )?;

    #[cfg(windows)]
    super::split_focus::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;
    #[cfg(windows)]
    super::navigation_state::install_native_navigation(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;

    #[cfg(windows)]
    crate::media::protocol::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        &navigation_registration_token,
    )?;
    #[cfg(windows)]
    crate::media::windows::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;
    #[cfg(windows)]
    crate::zoom::windows::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;
    #[cfg(windows)]
    crate::downloads::native::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;

    #[cfg(windows)]
    crate::site_permissions::requests::install_and_navigate(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
        initial_target,
    )?;
    Ok(())
}

/// Navigates the persistent child WebView to the specified URL without holding any state locks.
pub fn navigate_content_webview(app: &AppHandle, target_url_str: &str) -> Result<String, String> {
    navigate_content_webview_with_search_engine(app, target_url_str, "google")
}

/// Navigates the persistent child WebView using a supported search engine for text queries.
pub fn navigate_content_webview_with_search_engine(
    app: &AppHandle,
    target_url_str: &str,
    search_engine: &str,
) -> Result<String, String> {
    let parsed_url = super::url_input::resolve_browser_input(app, target_url_str, search_engine)?;
    let resolved_str = parsed_url.to_string();
    let state = app.state::<BrowserState>();
    let enabled = *state.shield_enabled.lock().map_err(|e| e.to_string())?;
    if should_block_url(&resolved_str, enabled) {
        return Err("Navigation blocked by the privacy shield".to_string());
    }

    active_content_webview(app)
        .ok_or("Active WebView was not found")?
        .navigate(parsed_url)
        .map_err(|e| format!("Navigation error: {e}"))?;

    Ok(resolved_str)
}

pub fn go_back_content_webview(app: &AppHandle) -> Result<(), String> {
    navigate_content_history(app, true)
}

pub fn go_forward_content_webview(app: &AppHandle) -> Result<(), String> {
    navigate_content_history(app, false)
}

fn navigate_content_history(app: &AppHandle, go_back: bool) -> Result<(), String> {
    let webview =
        active_content_webview(app).ok_or_else(|| "Content WebView was not found".to_string())?;

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
