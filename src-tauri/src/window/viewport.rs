use super::webview_manager::{
    BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, CONTENT_WEBVIEW_PREFIX, WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use crate::state::{
    browser_state::{BrowserState, ViewportBounds},
    tab_model::webview_label_for_tab,
};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect};

/// Geometry updates do not reconcile the tab lifecycle or acquire the database/operation lock.
pub fn resize_visible_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let bounds = *state.bounds.lock().map_err(|e| e.to_string())?;
    #[cfg(not(target_os = "linux"))]
    let rect = Rect {
        position: LogicalPosition::new(bounds.x, bounds.y).into(),
        size: LogicalSize::new(bounds.width, bounds.height).into(),
    };
    sync_content_geometry(app, state)?;
    if *state
        .workspace_switcher_open
        .lock()
        .map_err(|e| e.to_string())?
    {
        #[cfg(target_os = "linux")]
        super::linux_layout::update_bounds(
            app,
            WORKSPACE_SWITCHER_WEBVIEW_LABEL,
            bounds.x as i32,
            bounds.y as i32,
            bounds.width as i32,
            bounds.height as i32,
            true,
        )?;
        #[cfg(not(target_os = "linux"))]
        if let Some(view) = app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL) {
            view.set_bounds(rect).map_err(|e| e.to_string())?;
        }
    }
    if *state
        .bookmark_context_menu_open
        .lock()
        .map_err(|e| e.to_string())?
    {
        if let (Some(_view), Some(window)) = (
            app.get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL),
            app.get_window("main"),
        ) {
            let size = window
                .inner_size()
                .map_err(|e| e.to_string())?
                .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
            #[cfg(target_os = "linux")]
            super::linux_layout::update_bounds(
                app,
                BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL,
                0,
                0,
                size.width as i32,
                size.height as i32,
                true,
            )?;
            #[cfg(not(target_os = "linux"))]
            _view
                .set_bounds(Rect {
                    position: LogicalPosition::new(0.0, 0.0).into(),
                    size: size.into(),
                })
                .map_err(|e| e.to_string())?;
        }
    }
    super::media_popover::resize(app)?;
    super::tools_popover::resize(app)?;
    super::downloads_popover::resize(app)?;
    super::quick_settings_popover::resize(app)?;
    super::privacy_popover::resize(app)?;
    super::permission_popover::resize(app)?;
    super::history_overlay::resize(app)?;
    super::zoom_popover::resize(app)?;
    super::tab_drag_preview::paint(app, state)
}

pub fn content_bounds(
    app: &AppHandle,
    state: &BrowserState,
) -> Result<Vec<(String, ViewportBounds)>, String> {
    match super::tab_drag_preview::visible_bounds(app, state)? {
        Some(bounds) => Ok(bounds),
        None => visible_bounds(state),
    }
}

/// Presentation changes reuse resident documents and never change tab lifecycle metadata.
pub fn sync_content_geometry(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let panes = content_bounds(app, state)?;
    let hidden = *state.is_overlay_open.lock().map_err(|e| e.to_string())?
        || *state
            .bookmark_context_menu_open
            .lock()
            .map_err(|e| e.to_string())?;
    for (label, _view) in app.webviews() {
        if !label.starts_with(CONTENT_WEBVIEW_PREFIX) {
            continue;
        }
        let pane = panes
            .iter()
            .find(|(id, _)| webview_label_for_tab(id) == label);
        if let Some((_, bounds)) = pane.filter(|_| !hidden) {
            #[cfg(target_os = "linux")]
            super::linux_layout::update_bounds(
                app,
                &label,
                bounds.x as i32,
                bounds.y as i32,
                bounds.width.max(100.0) as i32,
                bounds.height.max(100.0) as i32,
                true,
            )?;
            #[cfg(not(target_os = "linux"))]
            {
                _view
                    .set_bounds(Rect {
                        position: LogicalPosition::new(bounds.x, bounds.y).into(),
                        size: LogicalSize::new(bounds.width.max(100.0), bounds.height.max(100.0))
                            .into(),
                    })
                    .map_err(|e| e.to_string())?;
                _view.show().map_err(|e| e.to_string())?;
            }
        } else {
            #[cfg(target_os = "linux")]
            super::linux_layout::set_visible(app, &label, false)?;
            #[cfg(not(target_os = "linux"))]
            _view.hide().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn visible_bounds(
    state: &BrowserState,
) -> Result<Vec<(String, crate::state::browser_state::ViewportBounds)>, String> {
    let bounds = *state.bounds.lock().map_err(|e| e.to_string())?;
    if let Some(layout) = state.display_split() {
        return Ok(crate::state::split_layout::pane_bounds(bounds, &layout).into());
    }
    Ok(vec![(
        state
            .active_tab_id
            .lock()
            .map_err(|e| e.to_string())?
            .clone(),
        bounds,
    )])
}
