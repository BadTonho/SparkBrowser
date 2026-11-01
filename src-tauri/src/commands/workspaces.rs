use crate::commands::tabs::create_tab;
use crate::database::{session_repo::save_session_tabs, workspace_repo};
use crate::state::browser_state::{BrowserState, DEFAULT_HOME_URL};
use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::WorkspaceItem;
use crate::window::webview_manager::{navigate_content_webview, sync_all_webviews};
use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Serialize)]
pub struct WorkspaceDeleteResult {
    pub workspaces: Vec<WorkspaceItem>,
    pub tabs: Vec<TabInfo>,
    pub active_workspace_id: String,
}

#[tauri::command]
pub async fn get_workspaces(state: State<'_, BrowserState>) -> Result<Vec<WorkspaceItem>, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Workspace database is not ready".to_string())?;
    workspace_repo::list_workspaces(conn)
}

#[tauri::command]
pub async fn get_active_workspace(state: State<'_, BrowserState>) -> Result<String, String> {
    Ok(state
        .active_workspace_id
        .lock()
        .map(|id| id.clone())
        .unwrap_or_else(|_| "ws-personal".to_string()))
}

#[tauri::command]
pub async fn switch_workspace(
    app: AppHandle,
    state: State<'_, BrowserState>,
    workspace_id: String,
) -> Result<Vec<TabInfo>, String> {
    let exists = state
        .workspaces
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .any(|workspace| workspace.id == workspace_id);
    if !exists {
        return Err("Workspace not found".to_string());
    }
    if let Ok(mut active_ws) = state.active_workspace_id.lock() {
        *active_ws = workspace_id.clone();
    }

    let existing_tab = {
        state.tabs.lock().ok().and_then(|mut tabs| {
            if let Some(found) = tabs.iter_mut().find(|t| t.workspace_id == workspace_id) {
                found.is_hibernated = false;
                found.last_active_epoch = TabInfo::current_epoch_secs();
                Some(found.clone())
            } else {
                None
            }
        })
    };

    if let Some(tab) = existing_tab {
        if let Ok(mut active_id) = state.active_tab_id.lock() {
            *active_id = tab.id;
        }
        let _ = navigate_content_webview(&app, &tab.url);
        sync_all_webviews(&app, &state)?;
        state.persist_session();

        let snapshot = state.snapshot_tabs();
        let _ = app.emit("tabs-updated", &snapshot);
        let _ = app.emit("content-url-changed", &tab.url);
        return Ok(snapshot);
    }

    create_tab(app, state, Some(DEFAULT_HOME_URL.to_string())).await
}

#[tauri::command]
pub async fn create_workspace(
    state: State<'_, BrowserState>,
    id: String,
    name: String,
    icon: String,
    accent_primary: String,
    accent_secondary: String,
) -> Result<Vec<WorkspaceItem>, String> {
    let workspace = WorkspaceItem {
        id,
        name,
        icon,
        accent_primary,
        accent_secondary,
    };
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Workspace database is not ready".to_string())?;
    workspace_repo::create_workspace(conn, &workspace)?;
    let list = workspace_repo::list_workspaces(conn)?;
    *state.workspaces.lock().map_err(|e| e.to_string())? = list.clone();
    Ok(list)
}

#[tauri::command]
pub async fn update_workspace(
    state: State<'_, BrowserState>,
    id: String,
    name: String,
    icon: String,
    accent_primary: String,
    accent_secondary: String,
) -> Result<Vec<WorkspaceItem>, String> {
    let workspace = WorkspaceItem {
        id,
        name,
        icon,
        accent_primary,
        accent_secondary,
    };
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Workspace database is not ready".to_string())?;
    workspace_repo::update_workspace(conn, &workspace)?;
    let list = workspace_repo::list_workspaces(conn)?;
    *state.workspaces.lock().map_err(|e| e.to_string())? = list.clone();
    Ok(list)
}

#[tauri::command]
pub async fn delete_workspace(
    app: AppHandle,
    state: State<'_, BrowserState>,
    workspace_id: String,
    action: String,
    target_workspace_id: Option<String>,
) -> Result<WorkspaceDeleteResult, String> {
    if workspace_id == "ws-personal" {
        return Err("The Personal workspace cannot be deleted".to_string());
    }
    if action != "move" && action != "close" {
        return Err("Choose whether to move or close the workspace tabs".to_string());
    }

    let current_workspaces = state.workspaces.lock().map_err(|e| e.to_string())?.clone();
    if !current_workspaces
        .iter()
        .any(|workspace| workspace.id == workspace_id)
    {
        return Err("Workspace not found".to_string());
    }
    let target_id = if action == "move" {
        let target_id =
            target_workspace_id.ok_or_else(|| "Choose a destination workspace".to_string())?;
        if target_id == workspace_id
            || !current_workspaces
                .iter()
                .any(|workspace| workspace.id == target_id)
        {
            return Err("Choose a different, existing destination workspace".to_string());
        }
        Some(target_id)
    } else {
        None
    };

    let active_workspace = state
        .active_workspace_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let was_active = active_workspace == workspace_id;
    let previous_active_tab = state
        .active_tab_id
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let mut next_tabs = state.tabs.lock().map_err(|e| e.to_string())?.clone();
    if action == "move" {
        for tab in next_tabs
            .iter_mut()
            .filter(|tab| tab.workspace_id == workspace_id)
        {
            tab.workspace_id = target_id.clone().expect("validated target");
        }
    } else {
        next_tabs.retain(|tab| tab.workspace_id != workspace_id);
    }

    let next_active_workspace = if was_active {
        target_id.unwrap_or_else(|| "ws-personal".to_string())
    } else {
        active_workspace
    };
    let mut next_active_tab = if was_active && action == "close" {
        next_tabs
            .iter()
            .rev()
            .find(|tab| tab.workspace_id == next_active_workspace)
            .map(|tab| tab.id.clone())
            .unwrap_or_default()
    } else if next_tabs.iter().any(|tab| {
        tab.id == previous_active_tab && (!was_active || tab.workspace_id == next_active_workspace)
    }) {
        previous_active_tab
    } else {
        next_tabs
            .iter()
            .rev()
            .find(|tab| tab.workspace_id == next_active_workspace)
            .map(|tab| tab.id.clone())
            .unwrap_or_default()
    };

    let mut next_counter = *state.next_tab_counter.lock().map_err(|e| e.to_string())?;
    if next_active_tab.is_empty() {
        let new_id = format!("tab-{next_counter}");
        next_counter += 1;
        next_tabs.push(TabInfo::new(
            new_id.clone(),
            DEFAULT_HOME_URL.to_string(),
            next_active_workspace.clone(),
            true,
        ));
        next_active_tab = new_id;
    }
    for tab in &mut next_tabs {
        tab.is_active = tab.id == next_active_tab;
    }
    let active_url = next_tabs
        .iter()
        .find(|tab| tab.id == next_active_tab)
        .map(|tab| tab.url.clone())
        .ok_or_else(|| "Could not select an active tab".to_string())?;

    let (updated_workspaces, committed_tabs) = {
        let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn: &mut Connection = db_guard
            .as_mut()
            .ok_or_else(|| "Workspace database is not ready".to_string())?;
        let transaction = conn.transaction().map_err(|e| e.to_string())?;
        save_session_tabs(&transaction, &next_tabs)?;
        workspace_repo::delete_workspace(&transaction, &workspace_id)?;
        let list = workspace_repo::list_workspaces(&transaction)?;
        transaction.commit().map_err(|e| e.to_string())?;
        (list, next_tabs)
    };

    *state.workspaces.lock().map_err(|e| e.to_string())? = updated_workspaces.clone();
    *state.tabs.lock().map_err(|e| e.to_string())? = committed_tabs.clone();
    *state
        .active_workspace_id
        .lock()
        .map_err(|e| e.to_string())? = next_active_workspace.clone();
    *state.active_tab_id.lock().map_err(|e| e.to_string())? = next_active_tab;
    *state.next_tab_counter.lock().map_err(|e| e.to_string())? = next_counter;

    if was_active {
        let _ = navigate_content_webview(&app, &active_url);
        let _ = app.emit("content-url-changed", &active_url);
    }
    sync_all_webviews(&app, &state)?;
    let tabs = state.snapshot_tabs();
    let _ = app.emit("tabs-updated", &tabs);

    Ok(WorkspaceDeleteResult {
        workspaces: updated_workspaces,
        tabs,
        active_workspace_id: next_active_workspace,
    })
}
