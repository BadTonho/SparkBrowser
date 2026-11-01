use super::internal_overlays::{
    handle_workspace_switcher_navigation, spawn_bookmark_context_menu_webview,
    spawn_workspace_switcher_webview,
};
#[cfg(windows)]
use super::keyboard_bridge::install_workspace_shortcut_accelerator;
use super::keyboard_bridge::{build_workspace_shortcut_script, WORKSPACE_SWITCHER_ESCAPE_SCRIPT};
use crate::privacy::adblock_engine::{build_content_shield_script, should_block_url};
use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::state::tab_model::{derive_title_from_url, webview_label_for_tab};
use tauri::{
    webview::WebviewBuilder, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect,
    Webview, WebviewUrl,
};

pub use super::content_lifecycle::reset_content_webviews;
pub use super::internal_overlays::{
    build_bookmark_context_menu_script, build_workspace_switcher_update_script,
    create_internal_bridge_token, BookmarkContextMenuAction,
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
    Ok(())
}

fn ensure_tab_webview(
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

    let webview_builder = WebviewBuilder::new(&label, WebviewUrl::External(initial_url))
        .initialization_script(build_content_shield_script(shield_enabled))
        .initialization_script(WORKSPACE_SWITCHER_ESCAPE_SCRIPT);

    let webview_builder = webview_builder.initialization_script(build_workspace_shortcut_script(
        &keyboard_bridge_token,
        native_alt_capture,
    ));

    let app_for_page_load = app.clone();
    let keyboard_token_for_load = keyboard_bridge_token.clone();
    let tab_for_load = tab_id.to_string();
    let label_for_load = label.clone();
    let webview_builder = webview_builder.on_page_load(move |webview, payload| {
        if payload.event() != tauri::webview::PageLoadEvent::Finished {
            return;
        }
        let Some(state) = app_for_page_load.try_state::<BrowserState>() else {
            return;
        };
        if !state
            .keyboard_bridge_tokens
            .lock()
            .map(|tokens| tokens.get(&label_for_load) == Some(&keyboard_token_for_load))
            .unwrap_or(false)
        {
            return;
        }
        let url = payload.url().to_string();
        if matches!(payload.url().scheme(), "http" | "https") {
            let title = derive_title_from_url(&url);
            if let Ok(mut tabs) = state.tabs.lock() {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == tab_for_load) {
                    tab.url = url.clone();
                    tab.title = title.clone();
                } else {
                    return;
                }
            }
            state.record_history_visit(&title, &url);
            if let Err(error) = state.persist_session() {
                eprintln!("Failed to persist completed navigation: {error}");
                let _ =
                    app_for_page_load.emit("browser-error", "Could not save the current session");
            }
            let _ = app_for_page_load.emit("tabs-updated", state.snapshot_tabs());
        }
        let Ok(shortcuts) = state.keyboard_shortcuts.lock() else {
            return;
        };
        if let Ok(serialized) = serde_json::to_string(&*shortcuts) {
            let _ = webview.eval(format!(
                "window.__veloceSetKeyboardShortcuts?.(\"{}\", {serialized});",
                keyboard_token_for_load
            ));
        }
    });

    let tab_for_nav = tab_id.to_string();
    let label_for_nav = label.clone();
    let webview_builder = webview_builder.on_navigation(move |nav_url| {
        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("workspace-switcher")
        {
            handle_workspace_switcher_navigation(&app_handle_for_nav, nav_url);
            return false;
        }

        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("keyboard-shortcut")
        {
            let params = nav_url.query_pairs().collect::<Vec<_>>();
            let request_token = params
                .iter()
                .find(|(key, _)| key == "token")
                .map(|(_, value)| value.as_ref());
            let sequence = params
                .iter()
                .find(|(key, _)| key == "sequence")
                .and_then(|(_, value)| value.parse::<u64>().ok());
            let token_matches = app_handle_for_nav
                .try_state::<BrowserState>()
                .and_then(|state| {
                    let is_active = state
                        .active_tab_id
                        .lock()
                        .map(|id| *id == tab_for_nav)
                        .unwrap_or(false);
                    state.keyboard_bridge_tokens.lock().ok().map(|tokens| {
                        is_active
                            && tokens.get(&label_for_nav).map(String::as_str) == request_token
                            && request_token == Some(keyboard_bridge_token.as_str())
                    })
                })
                .unwrap_or(false);
            let focused = app_handle_for_nav
                .get_webview_window("main")
                .and_then(|window| window.is_focused().ok())
                .unwrap_or(false);

            if token_matches && focused {
                let event_type = nav_url.path().trim_matches('/');
                match event_type {
                    "alt-down" | "alt-up" | "alt-cancel" => {
                        let _ = app_handle_for_nav.emit(
                            "workspace-shortcut-event",
                            serde_json::json!({ "type": event_type }),
                        );
                    }
                    "action" => {
                        let action = params
                            .iter()
                            .find(|(key, _)| key == "action")
                            .map(|(_, value)| value.as_ref());
                        let key = params
                            .iter()
                            .find(|(key, _)| key == "key")
                            .map(|(_, value)| value.as_ref());
                        if let Some(action) = action.filter(|action| {
                            matches!(
                                *action,
                                "switch-tabs"
                                    | "switch-workspaces"
                                    | "move-tab-up"
                                    | "move-tab-up-ctrl-shift"
                                    | "move-tab-down"
                                    | "move-tab-down-ctrl-shift"
                                    | "new-tab"
                                    | "navigate-back"
                                    | "navigate-forward"
                                    | "command-bar"
                                    | "edit-url"
                                    | "focus-address"
                                    | "sidebar"
                                    | "zen"
                                    | "history"
                                    | "downloads"
                                    | "settings"
                                    | "close-tab"
                                    | "reload-tab"
                            )
                        }) {
                            let _ = app_handle_for_nav.emit(
                                "browser-shortcut-action",
                                serde_json::json!({ "action": action, "key": key }),
                            );
                        }
                    }
                    _ => {}
                }
            } else if !focused {
                let _ = app_handle_for_nav.emit("workspace-shortcut-cancel", ());
            }

            if let Some(sequence) = sequence {
                if let Some(webview) = app_handle_for_nav.get_webview(&label_for_nav) {
                    let _ =
                        webview.eval(format!("window.__veloceWorkspaceShortcutAck({sequence});"));
                }
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
    let parsed_url = resolve_input_to_url_with_search_engine(target_url_str, search_engine)?;
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

/// Synchronizes visibility and bounds of the child WebView.
pub fn sync_all_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let bounds = state.bounds.lock().map(|b| *b).unwrap_or_default();
    let overlay_open = state.is_overlay_open.lock().map(|o| *o).unwrap_or(false);
    let workspace_switcher_open = state
        .workspace_switcher_open
        .lock()
        .map(|open| *open)
        .unwrap_or(false);
    let bookmark_context_menu_open = state
        .bookmark_context_menu_open
        .lock()
        .map(|open| *open)
        .unwrap_or(false);

    let tabs: std::collections::HashMap<_, _> = state
        .snapshot_tabs()
        .into_iter()
        .map(|tab| (webview_label_for_tab(&tab.id), tab))
        .collect();
    for (label, webview) in app.webviews() {
        if !label.starts_with(CONTENT_WEBVIEW_PREFIX) {
            continue;
        }
        let tab = tabs.get(&label);
        match tab {
            None
            | Some(crate::state::tab_model::TabInfo {
                is_hibernated: true,
                ..
            }) => {
                webview.close().map_err(|e| e.to_string())?;
                state
                    .keyboard_bridge_tokens
                    .lock()
                    .map_err(|e| e.to_string())?
                    .remove(&label);
            }
            Some(tab)
                if tab.is_active
                    && !overlay_open
                    && !workspace_switcher_open
                    && !bookmark_context_menu_open =>
            {
                webview
                    .set_bounds(Rect {
                        position: LogicalPosition::new(bounds.x, bounds.y).into(),
                        size: LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0))
                            .into(),
                    })
                    .map_err(|e| e.to_string())?;
                webview.show().map_err(|e| e.to_string())?;
            }
            Some(_) => webview.hide().map_err(|e| e.to_string())?,
        }
    }

    if let Some(webview) = app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL) {
        if workspace_switcher_open {
            let _ = webview.set_bounds(Rect {
                position: LogicalPosition::new(bounds.x, bounds.y).into(),
                size: LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)).into(),
            });
            let _ = webview.show();
        } else {
            let _ = webview.hide();
        }
    }

    if let Some(webview) = app.get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL) {
        if bookmark_context_menu_open {
            if let Some(window) = app.get_webview_window("main") {
                let scale_factor = window.scale_factor().unwrap_or(1.0);
                let logical_size = window
                    .inner_size()
                    .map(|size| size.to_logical::<f64>(scale_factor))
                    .unwrap_or_else(|_| LogicalSize::new(1200.0, 800.0));
                let _ = webview.set_bounds(Rect {
                    position: LogicalPosition::new(0.0, 0.0).into(),
                    size: logical_size.into(),
                });
            }
            let _ = webview.show();
        } else {
            let _ = webview.hide();
        }
    }

    Ok(())
}
