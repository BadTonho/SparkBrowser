use crate::state::{
    browser_state::{BrowserState, ViewportBounds},
    split_docking::{self, DockContext, DockSide},
    tab_model::webview_label_for_tab,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

mod geometry;
mod surface;
pub use surface::hide as hide_surface;

#[derive(Default)]
pub struct DragPreview(Mutex<PreviewState>);

#[derive(Default)]
struct PreviewState {
    revision: u64,
    paint_revision: u64,
    presentation: Option<Presentation>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    tab_id: String,
    side: Option<DockSide>,
    context: DockContext,
    label: String,
    locale: String,
    pointer: Pointer,
}

#[derive(Clone, Deserialize, Serialize)]
struct Pointer {
    x: f64,
    y: f64,
}

pub fn blocked(state: &BrowserState) -> bool {
    state
        .surfaces()
        .is_overlay_open()
        .lock()
        .map_or(true, |value| *value)
        || state
            .surfaces()
            .workspace_switcher_open()
            .lock()
            .map_or(true, |value| *value)
        || state
            .surfaces()
            .bookmark_context_menu_open()
            .lock()
            .map_or(true, |value| *value)
}

pub fn active(app: &AppHandle) -> bool {
    app.try_state::<DragPreview>()
        .is_some_and(|service| service.0.lock().map_or(true, |s| s.presentation.is_some()))
}

/// Forget presentation only. A successful drop reconciles before hiding the surface.
pub fn clear(app: &AppHandle) -> Result<bool, String> {
    let service = app.state::<DragPreview>();
    let mut preview = service.0.lock().map_err(|e| e.to_string())?;
    let changed = preview.presentation.take().is_some();
    if changed {
        preview.paint_revision = preview.paint_revision.saturating_add(1);
    }
    Ok(changed)
}

pub(super) fn paint_is_current(app: &AppHandle, revision: u64) -> bool {
    app.try_state::<DragPreview>().is_some_and(|service| {
        service.0.lock().is_ok_and(|preview| {
            preview.presentation.is_some() && preview.paint_revision == revision
        })
    })
}

/// Restore current committed geometry without waking, navigating or closing a page.
pub fn cancel(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let result = clear(app).and_then(|changed| {
        if changed {
            super::viewport::resize_visible_webviews(app, state)?;
        }
        Ok(())
    });
    hide_surface(app);
    result
}

fn current(app: &AppHandle, state: &BrowserState) -> Result<Option<(u64, Presentation)>, String> {
    let Some(service) = app.try_state::<DragPreview>() else {
        return Ok(None);
    };
    let preview = service.0.lock().map_err(|e| e.to_string())?;
    let Some(presentation) = preview.presentation.clone() else {
        return Ok(None);
    };
    let revision = preview.revision;
    drop(preview);
    if blocked(state)
        || app.state::<crate::shutdown::ShutdownService>().blocked()
        || split_docking::validate_context(state, &presentation.context, &presentation.tab_id)
            .is_err()
    {
        clear(app)?;
        hide_surface(app);
        return Ok(None);
    }
    Ok(Some((revision, presentation)))
}

fn geometry(
    state: &BrowserState,
    presentation: &Presentation,
) -> Result<Option<geometry::DockGeometry>, String> {
    presentation
        .side
        .map(|side| {
            let layout =
                split_docking::plan(state, &presentation.context, &presentation.tab_id, side)?;
            let bounds = state.surfaces().viewport(
                &state
                    .window_of_tab(&presentation.context.active_tab_id)
                    .unwrap_or_else(|_| crate::state::tab_model::MAIN_WINDOW_ID.to_string()),
            )?;
            Ok(geometry::DockGeometry::new(bounds, &layout, side))
        })
        .transpose()
}

fn resident(app: &AppHandle, state: &BrowserState, id: &str) -> bool {
    state
        .snapshot_tabs()
        .iter()
        .any(|tab| tab.id == id && !tab.is_hibernated)
        && app.get_webview(&webview_label_for_tab(id)).is_some()
}

/// None uses ordinary geometry; Some(empty) reserves both panes without loading a site.
pub fn visible_bounds(
    app: &AppHandle,
    state: &BrowserState,
) -> Result<Option<Vec<(String, ViewportBounds)>>, String> {
    let Some((_, presentation)) = current(app, state)? else {
        return Ok(None);
    };
    let Some(geometry) = geometry(state, &presentation)? else {
        return Ok(None);
    };
    Ok(Some(if resident(app, state, &geometry.companion.0) {
        vec![geometry.companion]
    } else {
        Vec::new()
    }))
}

pub fn paint(app: &AppHandle, state: &BrowserState) -> Result<(), String> {
    let Some((_, presentation)) = current(app, state)? else {
        return Ok(());
    };
    // Resizes and residency changes can repaint without a new pointer request.
    let service = app.state::<DragPreview>();
    let revision = {
        let mut preview = service.0.lock().map_err(|e| e.to_string())?;
        preview.paint_revision = preview.paint_revision.saturating_add(1);
        preview.paint_revision
    };
    let geometry = geometry(state, &presentation)?;
    let pane = geometry.as_ref().map(|g| g.destination);
    let placeholder = geometry
        .as_ref()
        .filter(|g| !resident(app, state, &g.companion.0))
        .map(|g| g.companion.1);
    let tab = state
        .snapshot_tabs()
        .into_iter()
        .find(|tab| tab.id == presentation.tab_id)
        .ok_or("Dragged tab not found")?;
    let host = url::Url::parse(&tab.url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_default();
    let window_id = state
        .window_of_tab(&presentation.context.active_tab_id)
        .unwrap_or_else(|_| crate::state::tab_model::MAIN_WINDOW_ID.to_string());
    surface::paint(
        app,
        &window_id,
        &serde_json::json!({
            "revision": revision, "label": presentation.label, "locale": presentation.locale,
            "pointer": presentation.pointer, "title": tab.title, "host": host, "pane": pane,
            "placeholder": placeholder,
        }),
    )
}

#[tauri::command]
pub async fn set_tab_drag_preview(
    app: AppHandle,
    state: tauri::State<'_, BrowserState>,
    revision: u64,
    presentation: Option<Presentation>,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    let service = app.state::<DragPreview>();
    {
        let mut latest = service.0.lock().map_err(|e| e.to_string())?;
        if revision <= latest.revision {
            return Ok(());
        }
        latest.revision = revision;
    }
    let Some(presentation) = presentation else {
        return cancel(&app, &state);
    };
    if blocked(&state) || app.state::<crate::shutdown::ShutdownService>().blocked() {
        cancel(&app, &state)?;
        return Err("A browser panel is open".into());
    }
    split_docking::validate_context(&state, &presentation.context, &presentation.tab_id)
        .inspect_err(|_| {
            let _ = cancel(&app, &state);
        })?;
    if !presentation.pointer.x.is_finite() || !presentation.pointer.y.is_finite() {
        cancel(&app, &state)?;
        return Err("Invalid drag pointer".into());
    }
    let result = (|| {
        let before = super::viewport::content_bounds(&app, &state)?;
        service.0.lock().map_err(|e| e.to_string())?.presentation = Some(presentation);
        if before != super::viewport::content_bounds(&app, &state)? {
            super::viewport::resize_visible_webviews(&app, &state)
        } else {
            paint(&app, &state)
        }
    })();
    result.inspect_err(|_| {
        let _ = cancel(&app, &state);
    })
}
