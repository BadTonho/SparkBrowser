use super::webview_manager::{
    BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL, CONTENT_WEBVIEW_PREFIX, WORKSPACE_SWITCHER_WEBVIEW_LABEL,
};
use crate::state::browser_state::{BrowserState, ViewportBounds};
#[cfg_attr(target_os = "linux", allow(unused_imports))]
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect};

/// Geometry updates do not reconcile the tab lifecycle or acquire the database/operation lock.
pub fn resize_visible_webviews(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let bounds = state.surfaces().main_viewport()?;
    #[cfg(not(target_os = "linux"))]
    let rect = Rect {
        position: LogicalPosition::new(bounds.x, bounds.y).into(),
        size: LogicalSize::new(bounds.width, bounds.height).into(),
    };
    sync_content_geometry(app, state)?;
    #[cfg(windows)]
    super::whatsapp_window::resize(app, bounds)?;
    if *state
        .surfaces()
        .workspace_switcher_open()
        .lock()
        .map_err(|e| e.to_string())?
    {
        #[cfg(target_os = "linux")]
        let bounds = state
            .surfaces()
            .viewport(&super::overlay_host::host_id(
                app,
                WORKSPACE_SWITCHER_WEBVIEW_LABEL,
            ))
            .unwrap_or(bounds);
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
        .surfaces()
        .bookmark_context_menu_open()
        .lock()
        .map_err(|e| e.to_string())?
    {
        if let (Some(_view), Some(window)) = (
            app.get_webview(BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL),
            super::overlay_host::window_of(app, BOOKMARK_CONTEXT_MENU_WEBVIEW_LABEL),
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
    super::suggestions_popover::resize(app)?;
    super::commands_popover::resize(app)?;
    super::resources_popover::resize(app)?;
    super::quick_settings_popover::resize(app)?;
    super::settings_overlay::resize(app)?;
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
    sync_changed_content_geometry(app, state, None)
}

pub(super) fn sync_changed_content_geometry(
    app: &AppHandle,
    state: &BrowserState,
    previous: Option<&[(String, ViewportBounds)]>,
) -> Result<(), String> {
    let mut panes = content_bounds(app, state)?;
    // A page in fullscreen fills its window and hides the other pages of that window.
    let fullscreen = super::fullscreen::all(app);
    let mut full_sizes = Vec::new();
    for (owner, window_id) in &fullscreen {
        let Some(window) = app.get_window(window_id) else {
            continue;
        };
        let size = window
            .inner_size()
            .map_err(|e| e.to_string())?
            .to_logical::<f64>(window.scale_factor().map_err(|e| e.to_string())?);
        full_sizes.push((owner.clone(), window_id.clone(), size));
    }
    panes.retain(|(id, _)| {
        let window = state.window_of_tab(id).unwrap_or_default();
        !full_sizes.iter().any(|(_, full, _)| full == &window)
    });
    // The tab and bookmark context menus are layers above the page, like the workspace switcher: they
    // must not hide the page, which used to turn the whole tab black while the menu was open.
    let hidden = *state
        .surfaces()
        .is_overlay_open()
        .lock()
        .map_err(|e| e.to_string())?;
    let previous = previous.filter(|_| fullscreen.is_empty() && !hidden);
    for (label, _view) in app.webviews() {
        let Some(tab_id) = label.strip_prefix(CONTENT_WEBVIEW_PREFIX) else {
            continue;
        };
        let full = full_sizes.iter().find(|(owner, _, _)| owner == &label);
        let pane = panes
            .iter()
            .find(|(id, _)| id == tab_id)
            .map(|(_, bounds)| *bounds);
        let pane = match full {
            Some((_, _, size)) => Some(crate::state::browser_state::ViewportBounds {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            }),
            None => pane.filter(|_| !hidden),
        };
        if let Some(previous) = previous {
            let old_bounds = previous
                .iter()
                .find(|(id, _)| id == tab_id)
                .map(|(_, bounds)| *bounds);
            if old_bounds == pane {
                continue;
            }
        }
        if let Some(bounds) = pane {
            // A layout change (a split opening, changing or closing) slides; anything else moves at once.
            let window_id = state.window_of_tab(tab_id).unwrap_or_default();
            let area = state
                .surfaces()
                .viewport(&window_id)
                .map(|view| (view.width, view.height))
                .unwrap_or_default();
            let slides = matches!(
                super::pane_animation::plan(app, &label, &window_id, area, bounds),
                super::pane_animation::Plan::Animated
            );
            #[cfg(target_os = "linux")]
            if !slides {
                super::linux_layout::update_bounds(
                    app,
                    &label,
                    bounds.x as i32,
                    bounds.y as i32,
                    bounds.width.max(100.0) as i32,
                    bounds.height.max(100.0) as i32,
                    true,
                )?;
            }
            #[cfg(not(target_os = "linux"))]
            {
                if !slides {
                    _view
                        .set_bounds(Rect {
                            position: LogicalPosition::new(bounds.x, bounds.y).into(),
                            size: LogicalSize::new(
                                bounds.width.max(100.0),
                                bounds.height.max(100.0),
                            )
                            .into(),
                        })
                        .map_err(|e| e.to_string())?;
                }
                _view.show().map_err(|e| e.to_string())?;
            }
        } else {
            super::pane_animation::forget(&label);
            #[cfg(target_os = "linux")]
            super::linux_layout::set_visible(app, &label, false)?;
            #[cfg(not(target_os = "linux"))]
            _view.hide().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Where every window shows its pages: the panes of all windows, in each window's coordinates.
pub fn visible_bounds(
    state: &BrowserState,
) -> Result<Vec<(String, crate::state::browser_state::ViewportBounds)>, String> {
    let mut panes = Vec::new();
    for window_id in state.session().window_ids() {
        panes.extend(visible_bounds_in(state, &window_id)?);
    }
    Ok(panes)
}

/// The panes one window shows.
pub fn visible_bounds_in(
    state: &BrowserState,
    window_id: &str,
) -> Result<Vec<(String, crate::state::browser_state::ViewportBounds)>, String> {
    let bounds = state.surfaces().viewport(window_id)?;
    if let Some(layout) = state.display_split_in(window_id) {
        return Ok(crate::state::split_layout::pane_bounds(bounds, &layout).into());
    }
    let window = state
        .session()
        .window(window_id)
        .ok_or("windows.not_found")?;
    let active = window
        .active_tab_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    Ok(vec![(active, bounds)])
}
