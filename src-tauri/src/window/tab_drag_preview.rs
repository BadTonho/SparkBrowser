use crate::state::{
    browser_state::BrowserState,
    split_docking::{self, DockContext, DockSide},
    split_layout::pane_bounds,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

mod surface;
pub use surface::hide;

#[derive(Default)]
pub struct DragPreview(pub Mutex<u64>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    tab_id: String,
    side: Option<DockSide>,
    context: DockContext,
    label: String,
    locale: String,
    pointer: Pointer,
}

#[derive(Deserialize, Serialize)]
struct Pointer {
    x: f64,
    y: f64,
}

pub fn blocked(state: &BrowserState) -> bool {
    state.is_overlay_open.lock().map_or(true, |value| *value)
        || state
            .workspace_switcher_open
            .lock()
            .map_or(true, |value| *value)
        || state
            .bookmark_context_menu_open
            .lock()
            .map_or(true, |value| *value)
}

#[tauri::command]
pub async fn set_tab_drag_preview(
    app: AppHandle,
    state: tauri::State<'_, BrowserState>,
    revision: u64,
    presentation: Option<Presentation>,
) -> Result<(), String> {
    let _operation = state.tab_operations.lock().await;
    let service = app.state::<DragPreview>();
    {
        let mut latest = service.0.lock().map_err(|e| e.to_string())?;
        if revision <= *latest {
            return Ok(());
        }
        *latest = revision;
    }
    let Some(presentation) = presentation else {
        hide(&app);
        return Ok(());
    };
    if blocked(&state) || app.state::<crate::shutdown::ShutdownService>().blocked() {
        hide(&app);
        return Err("A browser panel is open".into());
    }
    split_docking::validate_context(&state, &presentation.context, &presentation.tab_id)
        .inspect_err(|_| hide(&app))?;
    if !presentation.pointer.x.is_finite() || !presentation.pointer.y.is_finite() {
        hide(&app);
        return Err("Invalid drag pointer".into());
    }
    let pane = if let Some(side) = presentation.side {
        let next = split_docking::plan(&state, &presentation.context, &presentation.tab_id, side)
            .inspect_err(|_| hide(&app))?;
        let bounds = *state.bounds.lock().map_err(|e| e.to_string())?;
        Some(pane_bounds(bounds, &next)[if side == DockSide::Left { 0 } else { 1 }].1)
    } else {
        None
    };
    let tab = state
        .snapshot_tabs()
        .into_iter()
        .find(|tab| tab.id == presentation.tab_id)
        .ok_or("Dragged tab not found")?;
    let host = url::Url::parse(&tab.url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_default();
    let payload = serde_json::json!({
        "revision": revision, "label": presentation.label, "locale": presentation.locale,
        "pointer": presentation.pointer, "title": tab.title, "host": host, "pane": pane,
    });
    surface::paint(&app, &payload)
}
