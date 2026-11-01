use super::webview_manager::{
    BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use crate::state::browser_state::BrowserState;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect};

/// Geometry updates do not reconcile the tab lifecycle or acquire the database/operation lock.
pub fn resize_visible_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let bounds = *state.bounds.lock().map_err(|e| e.to_string())?;
    let rect = Rect {
        position: LogicalPosition::new(bounds.x, bounds.y).into(),
        size: LogicalSize::new(bounds.width, bounds.height).into(),
    };
    for (id, bounds) in visible_bounds(state)? {
        if let Some(view) = app.get_webview(&crate::state::tab_model::webview_label_for_tab(&id)) {
            view.set_bounds(Rect {
                position: LogicalPosition::new(bounds.x, bounds.y).into(),
                size: LogicalSize::new(bounds.width, bounds.height).into(),
            })
            .map_err(|e| e.to_string())?;
        }
    }
    if *state
        .workspace_switcher_open
        .lock()
        .map_err(|e| e.to_string())?
    {
        if let Some(view) = app.get_webview(WORKSPACE_SWITCHER_WEBVIEW_LABEL) {
            view.set_bounds(rect).map_err(|e| e.to_string())?;
        }
    }
    if *state
        .bookmark_context_menu_open
        .lock()
        .map_err(|e| e.to_string())?
    {
        if let (Some(view), Some(window)) = (
            app.get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL),
            app.get_window("main"),
        ) {
            let size = window
                .inner_size()
                .map_err(|e| e.to_string())?
                .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
            view.set_bounds(Rect {
                position: LogicalPosition::new(0.0, 0.0).into(),
                size: size.into(),
            })
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn visible_bounds(
    state: &BrowserState,
) -> Result<Vec<(String, crate::state::browser_state::ViewportBounds)>, String> {
    let bounds = *state.bounds.lock().map_err(|e| e.to_string())?;
    if let Some(layout) = state.current_split() {
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
