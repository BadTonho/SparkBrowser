use super::webview_manager::{
    ensure_active_content_webview, BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, CONTENT_WEBVIEW_PREFIX,
    WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use crate::state::browser_state::BrowserState;
use crate::state::tab_model::webview_label_for_tab;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect};

/// Prepare fallback metadata before persistence; no document is closed here.
pub fn prepare_content_reset(state: &BrowserState, enabled: bool) -> Result<(), String> {
    state.prepare_content_reset(enabled)
}

/// Closes a content WebView and retires every stateful session bound to it.
pub(super) fn close_and_clean(app: &AppHandle, label: &str) -> Result<(), String> {
    super::fullscreen::exit(app, label, true);
    let state = app.state::<BrowserState>();
    #[cfg(windows)]
    state.surfaces().content_bootstrap().retire(label)?;
    crate::site_permissions::cancel_view(app, label);
    crate::downloads::native::cancel_view(app, label);
    #[cfg(any(windows, target_os = "linux"))]
    app.run_on_main_thread({
        let label = label.to_owned();
        {
            let app = app.clone();
            move || crate::media::protocol::clear(&app, &label)
        }
    })
    .map_err(|e| e.to_string())?;
    if let Some(webview) = app.get_webview(label) {
        #[cfg(target_os = "linux")]
        super::linux_layout::remove_child(app, label);
        webview.close().map_err(|e| e.to_string())?;
    }
    super::new_tab::revoke(app, label);
    app.state::<super::settings_tab::SettingsTabSessions>()
        .revoke(label);
    state.surfaces().content_profiles().forget(label)?;
    state
        .security()
        .keyboard_bridge_sequences()
        .lock()
        .map_err(|_| "security.state_unavailable")?
        .remove(label);
    state
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .map_err(|e| e.to_string())?
        .remove(label);
    state
        .security()
        .shield_script_ids()
        .lock()
        .map_err(|e| e.to_string())?
        .remove(label);
    Ok(())
}

/// Drops a tab's WebView so the next sync recreates it inside the folder of its
/// current workspace. No document keeps writing into a previous data directory.
pub fn close_tab_webview(app: &AppHandle, tab_id: &str) -> Result<(), String> {
    let label = webview_label_for_tab(tab_id);
    if app.get_webview(&label).is_some() {
        close_and_clean(app, &label)
    } else {
        Ok(())
    }
}

/// Startup requests wait for native handlers; later requests reuse the same document.
pub fn navigate_tab_document(
    app: &AppHandle,
    tab_id: &str,
    target: tauri::Url,
) -> Result<(), String> {
    let label = webview_label_for_tab(tab_id);
    let view = app
        .get_webview(&label)
        .ok_or("Active WebView was not found")?;
    #[cfg(windows)]
    return app
        .state::<BrowserState>()
        .surfaces()
        .content_bootstrap()
        .navigate(&label, target.as_str(), |url| {
            view.navigate(tauri::Url::parse(url).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        });
    #[cfg(not(windows))]
    view.navigate(target).map_err(|e| e.to_string())
}

/// Rebuild views only after their fallback metadata has been committed.
pub fn reset_content_webviews(app: &AppHandle, _state: &BrowserState) -> Result<(), String> {
    for label in app
        .webviews()
        .keys()
        .filter(|label| label.starts_with(CONTENT_WEBVIEW_PREFIX))
    {
        close_and_clean(app, label)?;
    }
    ensure_active_content_webview(app)
}

/// Repeated reconciliation reuses an existing document without navigating it.
pub(super) fn ensure_tab_webview(
    app: &AppHandle,
    id: &str,
    url: &str,
    bounds: crate::state::browser_state::ViewportBounds,
) -> Result<bool, String> {
    let label = webview_label_for_tab(id);
    let profile = super::workspace_data::content_directory(app, id)?;
    let state = app.state::<BrowserState>();
    if app.get_webview(&label).is_some() {
        if state
            .surfaces()
            .content_profiles()
            .matches(&label, &profile)?
        {
            return Ok(false);
        }
        close_and_clean(app, &label)?;
    }
    super::content_factory::create_tab_webview(app, id, url, bounds, profile.clone())?;
    state
        .surfaces()
        .content_profiles()
        .remember(&label, profile)?;
    Ok(true)
}

/// Synchronizes visibility and bounds of the child WebView.
pub fn sync_all_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    sync_webviews_after_selection(app, state, None)
}

pub(crate) fn sync_webviews_after_selection(
    app: &AppHandle,
    state: &BrowserState,
    mut previous: Option<&[(String, crate::state::browser_state::ViewportBounds)]>,
) -> Result<(), String> {
    state.refresh_visible_tabs()?;
    // Only committed pages may be created; drag companions must already be resident.
    let pane_bounds = super::viewport::visible_bounds(state)?;
    let tabs = state.snapshot_tabs();
    for (id, bounds) in &pane_bounds {
        let tab = tabs
            .iter()
            .find(|tab| &tab.id == id)
            .ok_or("Visible tab not found")?;
        // A window of the last session that has not reopened yet gets its pages when it does.
        if app.get_window(&tab.window_id).is_none() {
            continue;
        }
        if ensure_tab_webview(app, id, &tab.url, *bounds)? {
            previous = None;
        }
    }
    let tabs = if state.enforce_resident_policy()? == 0 {
        tabs
    } else {
        state.snapshot_tabs()
    };
    let bounds = state.surfaces().main_viewport().unwrap_or_default();
    let workspace_switcher_open = state
        .surfaces()
        .workspace_switcher_open()
        .lock()
        .map(|open| *open)
        .unwrap_or(false);
    let bookmark_context_menu_open = state
        .surfaces()
        .bookmark_context_menu_open()
        .lock()
        .map(|open| *open)
        .unwrap_or(false);

    let tabs: std::collections::HashMap<_, _> =
        tabs.iter().map(|tab| (tab.id.as_str(), tab)).collect();
    for (label, _webview) in app.webviews() {
        let Some(id) = label.strip_prefix(CONTENT_WEBVIEW_PREFIX) else {
            continue;
        };
        let tab = tabs.get(id).copied();
        match tab {
            None
            | Some(crate::state::tab_model::TabInfo {
                is_hibernated: true,
                ..
            }) => {
                close_and_clean(app, &label)?;
            }
            Some(tab) => {
                let profile = super::workspace_data::content_directory(app, &tab.id)?;
                if !state
                    .surfaces()
                    .content_profiles()
                    .matches(&label, &profile)?
                {
                    close_and_clean(app, &label)?;
                }
            }
        }
    }
    let geometry = super::viewport::sync_changed_content_geometry(app, state, previous);
    if geometry.is_err() && previous.is_some() {
        super::viewport::sync_content_geometry(app, state)?;
    } else {
        geometry?;
    }

    if let Some(webview) = app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL) {
        let bounds = state
            .surfaces()
            .viewport(&super::overlay_host::host_id(
                app,
                WORKSPACE_SWITCHER_WEBVIEW_LABEL,
            ))
            .unwrap_or(bounds);
        if workspace_switcher_open {
            crate::error_log::ignored(webview.set_bounds(Rect {
                position: LogicalPosition::new(bounds.x, bounds.y).into(),
                size: LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0)).into(),
            }));
            crate::error_log::ignored(crate::window::show_overlay_webview(&webview));
        } else {
            crate::error_log::ignored(webview.hide());
        }
    }

    if let Some(webview) = app.get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL) {
        if bookmark_context_menu_open {
            if let Some(window) =
                super::overlay_host::window_of(app, BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL)
            {
                let scale_factor = window.scale_factor().unwrap_or(1.0);
                let logical_size = window
                    .inner_size()
                    .map(|size| size.to_logical::<f64>(scale_factor))
                    .unwrap_or_else(|_| LogicalSize::new(1200.0, 800.0));
                crate::error_log::ignored(webview.set_bounds(Rect {
                    position: LogicalPosition::new(0.0, 0.0).into(),
                    size: logical_size.into(),
                }));
            }
            crate::error_log::ignored(crate::window::show_overlay_webview(&webview));
        } else {
            crate::error_log::ignored(webview.hide());
        }
    }

    super::media_popover::sync(app, state)?;
    super::tools_popover::sync(app, state)?;
    super::downloads_popover::sync(app, state)?;
    super::suggestions_popover::sync(app, state)?;
    super::commands_popover::sync(app, state)?;
    super::resources_popover::sync(app, state)?;
    super::quick_settings_popover::sync(app, state)?;
    super::settings_overlay::sync(app, state)?;
    super::privacy_popover::sync(app, state)?;
    super::permission_popover::sync(app, state)?;
    super::history_overlay::sync(app, state)?;
    super::zoom_popover::sync(app, state)?;
    super::tab_drag_preview::paint(app, state)
}
