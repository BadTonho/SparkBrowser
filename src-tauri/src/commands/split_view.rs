use crate::state::{
    browser_state::{BrowserState, TabSnapshot},
    split_layout::SplitLayout,
    tab_checkpoint::TabCheckpoint,
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
    TabCheckpoint::commit(&state, || {
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
        Ok(())
    })?;
    *state.split_preview.lock().map_err(|e| e.to_string())? = None;
    super::tabs::reconcile_committed(&app, &state);
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
    let result = TabCheckpoint::commit(&state, || update_ratio(&state, &key, ratio));
    state.clear_split_preview(&key)?;
    if let Err(error) = result {
        if let Err(resize_error) = crate::window::viewport::resize_visible_webviews(&app, &state) {
            let _ = app.emit_to(
                "main",
                "browser-error",
                format!("Could not restore split geometry: {resize_error}"),
            );
        }
        return Err(error);
    }
    super::tabs::reconcile_committed(&app, &state);
    let snapshot = state.snapshot_packet();
    let _ = app.emit("tabs-updated", &snapshot);
    Ok(snapshot)
}
