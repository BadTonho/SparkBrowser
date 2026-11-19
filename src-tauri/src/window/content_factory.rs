use super::keyboard_bridge::build_workspace_shortcut_script;
#[cfg(windows)]
use super::keyboard_bridge::install_workspace_shortcut_accelerator;
use super::webview_manager::{
    create_internal_bridge_token, resolve_input_to_url_with_search_engine,
};
use crate::privacy::adblock_engine::{
    build_content_shield_script_with_exceptions, should_block_url,
};
use crate::state::browser_state::{BrowserState, ViewportBounds};
use crate::state::tab_model::webview_label_for_tab;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl};

pub(super) fn create_tab_webview(
    app: &AppHandle,
    tab_id: &str,
    initial_url_str: &str,
    bounds: ViewportBounds,
    profile: Option<std::path::PathBuf>,
) -> Result<(), String> {
    let label = webview_label_for_tab(tab_id);
    if app.get_webview(&label).is_some() {
        return Ok(());
    }

    let window_id = app
        .try_state::<BrowserState>()
        .and_then(|state| state.window_of_tab(tab_id).ok())
        .unwrap_or_else(|| crate::state::tab_model::MAIN_WINDOW_ID.to_string());
    let main_window = app
        .get_window(&window_id)
        .ok_or_else(|| format!("Window {window_id} not found"))?;

    let logical_target = resolve_input_to_url_with_search_engine(initial_url_str, "google")?;
    let initial_url = super::new_tab::physical_url(app, logical_target.as_str())?;
    let shield_enabled = app
        .try_state::<BrowserState>()
        .and_then(|s| s.policies().shield_enabled().lock().ok().map(|v| *v))
        .unwrap_or(true);
    if crate::privacy::adblock_engine::site_is_protected(
        &app.state::<BrowserState>(),
        initial_url.as_str(),
    ) && should_block_url(initial_url.as_str(), shield_enabled)
    {
        return Err("Navigation blocked by the privacy shield".to_string());
    }
    let keyboard_bridge_token = create_internal_bridge_token();
    if let Some(state) = app.try_state::<BrowserState>() {
        if let Ok(mut tokens) = state.security().keyboard_bridge_tokens().lock() {
            tokens.insert(label.clone(), keyboard_bridge_token.clone());
            if let Ok(mut sequences) = state.security().keyboard_bridge_sequences().lock() {
                sequences.remove(&label);
            }
        }
    }
    #[cfg(windows)]
    let native_alt_capture = true;
    #[cfg(not(windows))]
    let native_alt_capture = false;

    let app_handle_for_nav = app.clone();

    let focused = app.state::<BrowserState>().is_active_tab(tab_id);
    #[cfg(windows)]
    let initial_target = initial_url.to_string();
    #[cfg(windows)]
    let initial_url = tauri::Url::parse("about:blank").expect("static blank URL");
    let webview_builder = super::create_webview_builder(&label, WebviewUrl::External(initial_url))
        .initialization_script(include_str!("content_bridge.js").replace(
            "__CONTENT_NATIVE__",
            if cfg!(windows) { "true" } else { "false" },
        ))
        .initialization_script(super::new_tab::initializer(
            app,
            &keyboard_bridge_token,
            tab_id,
        )?)
        .initialization_script(
            [
                // WebKitGTK has no runtime bindings; the page reaches the host through a script
                // message handler registered by `media::protocol`.
                #[cfg(target_os = "linux")]
                "Object.defineProperty(window,'__sparkMediaSignal',{value:s=>window.webkit.messageHandlers.sparkMedia.postMessage(String(s))});",
                #[cfg(not(target_os = "linux"))]
                "",
                include_str!("../media/controller.js"),
            ]
            .concat()
                .replace(
                    "__MEDIA_TOKEN__",
                    &serde_json::to_string(&keyboard_bridge_token).expect("token serializable"),
                )
                .replace("__PIP_PLAYER__", &crate::media::pip::player_script())
                .replace(
                    "__MEDIA_ENGINE_ECHO__",
                    if cfg!(target_os = "linux") { "true" } else { "false" },
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
        .initialization_script(crate::privacy::auto_skip::initializer(
            &keyboard_bridge_token,
        ))
        .initialization_script(crate::privacy::forced_dark::initializer(
            &keyboard_bridge_token,
            crate::privacy::forced_dark::effective(
                app.state::<BrowserState>().policies().forced_dark_enabled(),
                super::new_tab::is_new_tab(initial_url_str)
                    || super::settings_tab::is_settings_url(initial_url_str),
            ),
        ))
        .zoom_hotkeys_enabled(false)
        .initialization_script(include_str!("page_activity.js").replace(
            "__ACTIVITY_TOKEN__",
            &serde_json::to_string(&keyboard_bridge_token).expect("token serializable"),
        ))
        .initialization_script(build_content_shield_script_with_exceptions(
            shield_enabled,
            &keyboard_bridge_token,
            &app.state::<BrowserState>()
                .policies()
                .shield_settings()
                .lock()
                .map_err(|e| e.to_string())?
                .exceptions,
        ))
        .initialization_script(crate::privacy::tracking_parameters::click_script(
            &*app
                .state::<BrowserState>()
                .policies()
                .tracking_settings()
                .lock()
                .map_err(|e| e.to_string())?,
            &keyboard_bridge_token,
        ))
        .initialization_script_for_all_frames(crate::privacy::gpc::initializer(
            &*app
                .state::<BrowserState>()
                .policies()
                .gpc_settings()
                .lock()
                .map_err(|e| e.to_string())?,
        ));

    // Opt-in workspace isolation gives every tab's WebView its own data folder.
    let webview_builder = match profile {
        Some(directory) => webview_builder.data_directory(directory),
        None => webview_builder,
    };

    #[cfg(not(windows))]
    let webview_builder = webview_builder.on_download(crate::downloads::native::handler(
        app.clone(),
        tab_id.to_string(),
    ));

    #[cfg(any(windows, target_os = "linux"))]
    let webview_builder = {
        let app = app.clone();
        let id = tab_id.to_string();
        let token = keyboard_bridge_token.clone();
        webview_builder.on_new_window(move |url, features| {
            let response = crate::media::pip_window::new_window(&app, &id, &token, &url, features);
            if matches!(&response, tauri::webview::NewWindowResponse::Deny) {
                crate::commands::tabs::open_popup_url(app.clone(), id.clone(), token.clone(), url);
            }
            response
        })
    };

    #[cfg(not(any(windows, target_os = "linux")))]
    let webview_builder = {
        let app = app.clone();
        let id = tab_id.to_string();
        let token = keyboard_bridge_token.clone();
        webview_builder.on_new_window(move |url, _features| {
            crate::commands::tabs::open_popup_url(app.clone(), id.clone(), token.clone(), url);
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

    let navigation_registration_token = keyboard_bridge_token.clone();
    let tab_for_nav = tab_id.to_string();
    let label_for_nav = label.clone();
    let webview_builder = webview_builder.on_navigation(move |nav_url| {
        if nav_url.scheme() == "spark-internal" {
            return super::content_bridge::handle(
                &app_handle_for_nav,
                &tab_for_nav,
                &label_for_nav,
                &keyboard_bridge_token,
                nav_url,
            );
        }

        let workspace_picker_is_open = app_handle_for_nav
            .try_state::<BrowserState>()
            .and_then(|state| {
                state
                    .surfaces()
                    .workspace_switcher_open()
                    .lock()
                    .ok()
                    .map(|open| *open)
            })
            .unwrap_or(false);

        let is_active = app_handle_for_nav
            .try_state::<BrowserState>()
            .map(|state| state.is_active_tab(&tab_for_nav))
            .unwrap_or(false);
        // Background pages must not cancel a picker held over the active tab.
        if is_active && !workspace_picker_is_open {
            if let Some(state) = app_handle_for_nav.try_state::<BrowserState>() {
                state
                    .surfaces()
                    .workspace_alt_pressed()
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                if let Ok(mut token) = state.surfaces().workspace_switcher_token().lock() {
                    *token = None;
                }
                if let Ok(mut open) = state.surfaces().workspace_switcher_open().lock() {
                    *open = false;
                }
            }
            crate::error_log::ignored(crate::security::emit_shell_for_tab(
                &app_handle_for_nav,
                &tab_for_nav,
                "workspace-shortcut-cancel",
                (),
            ));
            crate::error_log::ignored(crate::security::emit_shell_for_tab(
                &app_handle_for_nav,
                &tab_for_nav,
                "workspace-switcher-dismissed",
                (),
            ));
        }

        if super::new_tab::is_document(&app_handle_for_nav, nav_url)
            || super::settings_tab::is_document(&app_handle_for_nav, nav_url)
        {
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
            let shield_on = state
                .policies()
                .shield_enabled()
                .lock()
                .map(|v| *v)
                .unwrap_or(true);

            #[cfg(target_os = "linux")]
            if shield_on
                && crate::privacy::adblock_engine::site_is_protected(&state, &new_url_str)
                && app_handle_for_nav
                    .state::<std::sync::Arc<crate::privacy::filter_lists::FilterLists>>()
                    .blocks(&new_url_str, &new_url_str, "document", "")
            {
                state.policies().record_blocked_request();
                return false;
            }

            if crate::privacy::adblock_engine::site_is_protected(&state, &new_url_str)
                && should_block_url(&new_url_str, shield_on)
            {
                state.policies().record_blocked_request();
                let stats = state.snapshot_privacy_stats();
                crate::error_log::ignored(crate::security::emit_shell(
                    &app_handle_for_nav,
                    "privacy-stats-updated",
                    &stats,
                ));
                return false;
            }
        }
        true
    });

    #[cfg(windows)]
    app.state::<BrowserState>()
        .surfaces()
        .content_bootstrap()
        .begin(&label, &navigation_registration_token, &initial_target)?;
    main_window
        .add_child(
            webview_builder,
            LogicalPosition::new(bounds.x, bounds.y),
            LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)),
        )
        .map_err(|e| format!("Failed to create content WebView: {}", e))?;

    #[cfg(target_os = "linux")]
    super::caret_browsing::apply_to_webview(
        app,
        &label,
        app.state::<BrowserState>()
            .policies()
            .caret_browsing_enabled(),
    )?;

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
    crate::privacy::windows_filter::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
    )?;

    // The settings page is the app's own and dark by default: it must not show white while it loads.
    #[cfg(windows)]
    if super::settings_tab::is_settings_url(initial_url_str) {
        super::windows_background::dark_until_painted(
            &app.get_webview(&label).ok_or("New WebView was not found")?,
        );
    }

    #[cfg(target_os = "linux")]
    crate::privacy::content_filter_linux::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
    )?;
    super::fullscreen::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
    )?;
    super::link_context_menu::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;

    #[cfg(any(windows, target_os = "linux"))]
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
    // The page starts with the speed limit and the network monitor of the resources panel, if they are on.
    crate::resources::webview_created(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
    );
    #[cfg(any(windows, target_os = "linux"))]
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
    #[cfg(target_os = "linux")]
    crate::media::protocol::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        &navigation_registration_token,
    )?;
    #[cfg(target_os = "linux")]
    crate::media::linux::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;
    #[cfg(windows)]
    crate::media::windows::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;
    #[cfg(target_os = "linux")]
    crate::zoom::linux::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        &navigation_registration_token,
    )?;
    #[cfg(windows)]
    crate::zoom::windows::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;
    #[cfg(target_os = "linux")]
    crate::downloads::native::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
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
    #[cfg(target_os = "linux")]
    super::favicon::install(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
        tab_id,
        &navigation_registration_token,
    )?;
    crate::extensions::platform::sync_new_webview(
        app,
        &app.get_webview(&label).ok_or("New WebView was not found")?,
    )?;
    #[cfg(target_os = "linux")]
    crate::site_permissions::linux::install(
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
    )?;
    Ok(())
}
