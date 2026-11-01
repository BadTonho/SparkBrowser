use crate::{
    state::{
        browser_state::{BrowserState, TabSnapshot},
        split_layout::SplitLayout,
    },
    window::webview_manager::sync_all_webviews,
};
use tauri::{AppHandle, Emitter, State};
#[tauri::command]
pub async fn set_split_view(
    app: AppHandle,
    state: State<'_, BrowserState>,
    left_tab_id: Option<String>,
    right_tab_id: Option<String>,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    let workspace = state
        .active_workspace_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let previous = state
        .split_layouts
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let previous_tabs = state.snapshot_tabs();
    let previous_active = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    match (left_tab_id, right_tab_id) {
        (Some(left), Some(right)) => {
            let active = state
                .active_tab_id
                .lock()
                .map_err(|e| e.to_string())?
                .clone();
            let layout = SplitLayout {
                workspace_id: workspace.clone(),
                left_tab_id: left.clone(),
                right_tab_id: right.clone(),
                ratio: 0.5,
                focused_tab_id: if active == right { right } else { left },
            };
            if !layout.valid(&state.snapshot_tabs()) {
                return Err("Choose two different tabs from the current workspace".into());
            }
            state
                .split_layouts
                .lock()
                .map_err(|e| e.to_string())?
                .insert(workspace, layout.clone());
            state.select_tab(&layout.focused_tab_id)?;
        }
        (None, None) => {
            state
                .split_layouts
                .lock()
                .map_err(|e| e.to_string())?
                .remove(&workspace);
        }
        _ => return Err("Both split tabs are required".into()),
    }
    state.refresh_visible_tabs()?;
    if let Err(error) = sync_all_webviews(&app, &state).and_then(|()| state.persist_session()) {
        *state.split_layouts.lock().map_err(|e| e.to_string())? = previous;
        *state.tabs.lock().map_err(|e| e.to_string())? = previous_tabs;
        state.select_tab(&previous_active)?;
        let _ = state.refresh_visible_tabs();
        let _ = sync_all_webviews(&app, &state);
        return Err(error);
    }
    let snapshot = state.snapshot_packet();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}
pub fn update_ratio(state: &BrowserState, key: &str, ratio: f64) -> Result<(), String> {
    if !ratio.is_finite() || !(0.2..=0.8).contains(&ratio) {
        return Err("Invalid split ratio".into());
    }
    let workspace = state
        .active_workspace_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let mut layouts = state.split_layouts.lock().map_err(|e| e.to_string())?;
    let layout = layouts
        .get_mut(&workspace)
        .filter(|layout| layout.key() == key)
        .ok_or("Split view changed")?;
    layout.ratio = ratio;
    Ok(())
}
#[tauri::command]
pub async fn commit_split_ratio(
    app: AppHandle,
    state: State<'_, BrowserState>,
    key: String,
    ratio: f64,
) -> Result<TabSnapshot, String> {
    let _operation = state.tab_operations.lock().await;
    update_ratio(&state, &key, ratio)?;
    crate::window::viewport::resize_visible_webviews(&app, &state)?;
    state.persist_session()?;
    let snapshot = state.snapshot_packet();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}
