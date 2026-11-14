use crate::state::{
    browser_state::{BrowserState, TabSnapshot},
    split_layout::SplitLayout,
    tab_checkpoint::TabCheckpoint,
};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn move_tab_in_split(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    tab_id: String,
    side: crate::state::split_docking::DockSide,
    before_id: Option<String>,
    context: crate::state::split_docking::DockContext,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    crate::window::tab_drag_preview::cancel(&app, &state)?;
    if crate::window::tab_drag_preview::blocked(&state)
        || app.state::<crate::shutdown::ShutdownService>().blocked()
    {
        return Err("A browser panel is open".into());
    }
    crate::state::split_transfer::commit(&state, &context, &tab_id, side, before_id.as_deref())?;
    super::tabs::reconcile_committed(&app, &state);
    let snapshot = crate::state::publish::publish_tabs(&app, &state, &window_id);
    let active = state
        .snapshot_tabs()
        .into_iter()
        .find(|tab| tab.is_active && tab.window_id == window_id)
        .ok_or("Active tab not found")?;
    let _ = crate::security::emit_shell_to(&app, &window_id, "content-url-changed", active.url);
    Ok(snapshot)
}

#[tauri::command]
pub async fn dock_tab_to_split(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    tab_id: String,
    side: crate::state::split_docking::DockSide,
    context: crate::state::split_docking::DockContext,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    let result = if crate::window::tab_drag_preview::blocked(&state)
        || app.state::<crate::shutdown::ShutdownService>().blocked()
    {
        Err("A browser panel is open".into())
    } else {
        crate::state::split_docking::commit(&state, &context, &tab_id, side)
    };
    let url = result.inspect_err(|_| {
        if let Err(error) = crate::window::tab_drag_preview::cancel(&app, &state) {
            let _ = crate::security::emit_shell(
                &app,
                "browser-error",
                format!("Could not restore split geometry: {error}"),
            );
        }
    })?;
    crate::window::tab_drag_preview::clear(&app)?;
    *state
        .session()
        .edit_split_preview()
        .lock()
        .map_err(|e| e.to_string())? = None;
    super::tabs::reconcile_committed(&app, &state);
    crate::window::tab_drag_preview::hide_surface(&app);
    let snapshot = crate::state::publish::publish_tabs(&app, &state, &window_id);
    crate::security::emit_shell_all(
        &app,
        "privacy-stats-updated",
        state.snapshot_privacy_stats(),
    );
    let _ = crate::security::emit_shell_to(&app, &window_id, "content-url-changed", url);
    if let Err(error) =
        crate::window::webview_manager::focus_active_content_webview(&app, &window_id)
    {
        let _ = crate::security::emit_shell(&app, "browser-error", error);
    }
    Ok(snapshot)
}
#[tauri::command]
pub async fn set_split_view(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    left_tab_id: Option<String>,
    right_tab_id: Option<String>,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let window = state
        .session()
        .window(&window_id)
        .ok_or("windows.not_found")?;
    let _operation = state.session().operation().await;
    let workspace = window
        .edit_active_workspace_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    TabCheckpoint::commit(&state, || {
        match (left_tab_id, right_tab_id) {
            (Some(left), Some(right)) => {
                let active = window
                    .edit_active_tab_id()
                    .lock()
                    .map_err(|e| e.to_string())?
                    .clone();
                let layout = SplitLayout {
                    left_tab_ids: Vec::new(),
                    right_tab_ids: Vec::new(),
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
                    .session()
                    .edit_split_layouts()
                    .lock()
                    .map_err(|e| e.to_string())?
                    .insert(workspace, layout.clone());
                state.select_tab(&layout.focused_tab_id)?;
            }
            (None, None) => {
                state
                    .session()
                    .edit_split_layouts()
                    .lock()
                    .map_err(|e| e.to_string())?
                    .remove(&workspace);
            }
            _ => return Err("Both split tabs are required".into()),
        }
        Ok(())
    })?;
    *state
        .session()
        .edit_split_preview()
        .lock()
        .map_err(|e| e.to_string())? = None;
    super::tabs::reconcile_committed(&app, &state);
    Ok(crate::state::publish::publish_tabs(
        &app, &state, &window_id,
    ))
}
pub fn update_ratio(
    state: &BrowserState,
    window_id: &str,
    key: &str,
    ratio: f64,
) -> Result<(), String> {
    if !ratio.is_finite() || !(0.2..=0.8).contains(&ratio) {
        return Err("Invalid split ratio".into());
    }
    let workspace = state
        .session()
        .window(window_id)
        .ok_or("windows.not_found")?
        .edit_active_workspace_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let mut layouts = state
        .session()
        .edit_split_layouts()
        .lock()
        .map_err(|e| e.to_string())?;
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
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    key: String,
    ratio: f64,
) -> Result<TabSnapshot, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let _operation = state.session().operation().await;
    let result = TabCheckpoint::commit(&state, || update_ratio(&state, &window_id, &key, ratio));
    state.clear_split_preview(&key)?;
    if let Err(error) = result {
        if let Err(resize_error) = crate::window::viewport::resize_visible_webviews(&app, &state) {
            let _ = crate::security::emit_shell(
                &app,
                "browser-error",
                format!("Could not restore split geometry: {resize_error}"),
            );
        }
        return Err(error);
    }
    super::tabs::reconcile_committed(&app, &state);
    Ok(crate::state::publish::publish_tabs(
        &app, &state, &window_id,
    ))
}
