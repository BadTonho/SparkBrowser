use super::internal_overlays::{
    spawn_bookmark_context_menu_webview, spawn_workspace_switcher_webview,
};
use super::keyboard_bridge::build_workspace_shortcut_script;
#[cfg(windows)]
use super::keyboard_bridge::install_workspace_shortcut_accelerator;
use super::navigation_state::update_navigation;
use crate::privacy::adblock_engine::{build_content_shield_script, should_block_url};
use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::state::tab_model::webview_label_for_tab;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Webview, WebviewUrl};

pub use super::content_focus::focus_active_content_webview;
pub use super::content_lifecycle::{
    close_tab_webview, prepare_content_reset, reset_content_webviews, sync_all_webviews,
};
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

    let logical_target = resolve_input_to_url_with_search_engine(initial_url_str, "google")?;
    let initial_url = super::new_tab::physical_url(app, logical_target.as_str())?;
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
            if let Ok(mut sequences) = state.keyboard_bridge_sequences.lock() {
                sequences.remove(&label);
            }
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
    let webview_builder = super::create_webview_builder(&label, WebviewUrl::External(initial_url))
        .initialization_script(super::new_tab::initializer(
            app,
            &keyboard_bridge_token,
            tab_id,
        )?)
        .initialization_script(
            include_str!("../media/controller.js")
                .replace(
                    "__MEDIA_TOKEN__",
                    &serde_json::to_string(&keyboard_bridge_token).expect("token serializable"),
                )
                .replace("__PIP_PLAYER__", &crate::media::pip::player_script())
                .replace(
                    "__MEDIA_VOLUME__",
                    &app.state::<crate::media::MediaService>()
                        .tab(tab_id)
                        .volume
                        .to_string(),
                ),
        )
        .focused(focused)
        .initialization_script(crate::privacy::auto_skip::initializer(
            &keyboard_bridge_token,
        ))
        .zoom_hotkeys_enabled(false)
        .initialization_script(include_str!("page_activity.js").replace(
            "__ACTIVITY_TOKEN__",
            &serde_json::to_string(&keyboard_bridge_token).expect("token serializable"),
        ))
        .initialization_script(build_content_shield_script(
            shield_enabled,
            &keyboard_bridge_token,
        ));

    // Opt-in workspace isolation gives every tab's WebView its own data folder.
    let webview_builder = match crate::window::workspace_data::content_directory(app, tab_id)? {
        Some(directory) => webview_builder.data_directory(directory),
        None => webview_builder,
    };

    #[cfg(not(windows))]
    let webview_builder = webview_builder.on_download(crate::downloads::native::handler(
        app.clone(),
        tab_id.to_string(),
    ));

    #[cfg(windows)]
    let webview_builder = {
        let app = app.clone();
        let id = tab_id.to_string();
        let token = keyboard_bridge_token.clone();
        webview_builder.on_new_window(move |url, features| {
            let response = crate::media::pip_window::new_window(&app, &id, &token, &url, features);
            if matches!(&response, tauri::webview::NewWindowResponse::Deny) {
                crate::commands::tabs::open_popup_url(app.clone(), url);
            }
            response
        })
    };

    #[cfg(not(windows))]
    let webview_builder = {
        let app = app.clone();
        webview_builder.on_new_window(move |url, _features| {
            crate::commands::tabs::open_popup_url(app.clone(), url);
            tauri::webview::NewWindowResponse::Deny
        })
    };

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
        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("new-tab") {
            super::new_tab::handle(
                &app_handle_for_nav,
                &tab_for_nav,
                &label_for_nav,
                &keyboard_bridge_token,
                nav_url,
            );
            return false;
        }
        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("page-activity") {
            if nav_url.path() != "/state" {
                return false;
            }
            let Some(values) = crate::security::bridge::query(
                nav_url,
                &["token", "document", "sequence", "dirty", "media"],
            ) else {
                return false;
            };
            let (Some(dirty), Some(media)) = (
                crate::security::bridge::boolean(values.get("dirty")),
                crate::security::bridge::boolean(values.get("media")),
            ) else {
                return false;
            };
            let (Some(document), Some(sequence)) = (
                values.get("document"),
                values.get("sequence").and_then(|v| v.parse::<u64>().ok()),
            ) else {
                return false;
            };
            if values.get("token").map(String::as_str) == Some(keyboard_bridge_token.as_str())
                && super::navigation_state::is_current_generation(
                    &app_handle_for_nav,
                    &label_for_nav,
                    &keyboard_bridge_token,
                )
                && app_handle_for_nav
                    .state::<BrowserState>()
                    .keyboard_bridge_sequences
                    .lock()
                    .is_ok_and(|mut states| {
                        states.admit(&format!("{label_for_nav}:activity"), document, sequence)
                    })
            {
                update_navigation(
                    &app_handle_for_nav,
                    &tab_for_nav,
                    &label_for_nav,
                    &keyboard_bridge_token,
                    |tab| {
                        tab.activity.dirty_form = dirty;
                        tab.activity.document_media = media;
                        tab.activity.refresh_media();
                    },
                );
            }
            return false;
        }
        if nav_url.scheme() == "veloce-internal" && nav_url.host_str() == Some("workspace-switcher")
        {
            // Workspace selection belongs only to the packaged picker surface.
            // Content Escape uses the authenticated keyboard channel instead.
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
            if !matches!(nav_url.path(), "" | "/") {
                return false;
            }
            if crate::security::bridge::query(nav_url, &["token"]).is_none() {
                return false;
            }
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
            let _ =
                crate::security::emit_shell(&app_handle_for_nav, "workspace-shortcut-cancel", ());
            let _ = crate::security::emit_shell(
                &app_handle_for_nav,
                "workspace-switcher-dismissed",
                (),
            );
        }

        if super::new_tab::is_document(&app_handle_for_nav, nav_url) {
            return true;
        }
        if !matches!(nav_url.scheme(), "http" | "https")
            || nav_url.host_str() == Some("tauri.localhost")
            || nav_url.origin()
                == crate::security::documents::asset_base(&app_handle_for_nav).origin()
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
                let _ = crate::security::emit_shell(
                    &app_handle_for_nav,
                    "privacy-stats-updated",
                    &stats,
                );
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

    #[cfg(target_os = "linux")]
    super::linux_layout::update_bounds(
        app,
        &label,
        bounds.x as i32,
        bounds.y as i32,
        bounds.width.max(100.0) as i32,
        bounds.height.max(100.0) as i32,
        focused,
    )?;

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
    crate::security::harden(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        false,
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
    super::favicon::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        &navigation_registration_token,
    )?;
    crate::extensions::platform::sync_new_webview(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
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
        .navigate(super::new_tab::physical_url(app, parsed_url.as_str())?)
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
