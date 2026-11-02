use super::webview_manager::{
    ensure_active_content_webview, BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, CONTENT_WEBVIEW_PREFIX,
    WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use crate::state::browser_state::BrowserState;
use crate::state::tab_model::webview_label_for_tab;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect};

/// Rebuild views when their immutable initialization policy changes.
pub fn reset_content_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    state
        .shield_script_ids
        .lock()
        .map_err(|e| e.to_string())?
        .clear();
    state
        .keyboard_bridge_tokens
        .lock()
        .map_err(|e| e.to_string())?
        .clear();
    for (label, view) in app.webviews() {
        if label.starts_with(CONTENT_WEBVIEW_PREFIX) {
            crate::site_permissions::cancel_view(app, &label);
            crate::downloads::native::cancel_view(app, &label);
            #[cfg(windows)]
            app.run_on_main_thread({
                let label = label.clone();
                move || crate::media::protocol::clear(&label)
            })
            .map_err(|e| e.to_string())?;
            view.close().map_err(|e| e.to_string())?;
        }
    }
    let active = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    for tab in state.tabs.lock().map_err(|e| e.to_string())?.iter_mut() {
        tab.is_hibernated = tab.id != active;
        if tab.id == active
            && crate::privacy::adblock_engine::should_block_url(
                &tab.url,
                *state.shield_enabled.lock().map_err(|e| e.to_string())?,
            )
        {
            tab.url = crate::state::browser_state::DEFAULT_HOME_URL.to_string();
        }
    }
    ensure_active_content_webview(app)
}

/// Synchronizes visibility and bounds of the child WebView.
pub fn sync_all_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    state.refresh_visible_tabs()?;
    let pane_bounds = super::viewport::visible_bounds(state)?;
    for (id, bounds) in &pane_bounds {
        let tab = state
            .snapshot_tabs()
            .into_iter()
            .find(|t| &t.id == id)
            .ok_or("Visible tab not found")?;
        super::webview_manager::ensure_tab_webview(app, id, &tab.url, *bounds)?;
    }
    crate::state::memory_policy::enforce_resident_limit(
        &mut state.tabs.lock().map_err(|e| e.to_string())?,
        state
            .resident_limit
            .load(std::sync::atomic::Ordering::Relaxed),
    );
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
                crate::site_permissions::cancel_view(app, &label);
                crate::downloads::native::cancel_view(app, &label);
                #[cfg(windows)]
                app.run_on_main_thread({
                    let label = label.clone();
                    move || crate::media::protocol::clear(&label)
                })
                .map_err(|e| e.to_string())?;
                webview.close().map_err(|e| e.to_string())?;
                state
                    .keyboard_bridge_tokens
                    .lock()
                    .map_err(|e| e.to_string())?
                    .remove(&label);
                state
                    .shield_script_ids
                    .lock()
                    .map_err(|e| e.to_string())?
                    .remove(&label);
            }
            // The transparent native workspace picker is layered above the pages.
            // Keep both split panes visible behind it, including its exit animation.
            Some(tab) if tab.is_visible && !overlay_open && !bookmark_context_menu_open => {
                let bounds = pane_bounds
                    .iter()
                    .find(|(id, _)| id == &tab.id)
                    .map(|(_, rect)| *rect)
                    .unwrap_or(bounds);
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
