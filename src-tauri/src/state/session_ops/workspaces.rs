use super::tabs::{create_tab_inner, reconcile_committed, switch_tab_inner};
use crate::database::{
    preferences_repo, session_repo::save_session_tabs, split_repo, tab_usage_repo, workspace_repo,
};
use crate::state::browser_state::{BrowserState, TabSnapshot, DEFAULT_HOME_URL};
use crate::state::tab_model::TabInfo;
use crate::state::workspace_model::WorkspaceItem;
use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Debug, Serialize)]
pub struct WorkspaceDeleteResult {
    pub workspaces: Vec<WorkspaceItem>,
    pub tabs: TabSnapshot,
    pub active_workspace_id: String,
}

#[tauri::command]
pub async fn get_workspaces(state: State<'_, BrowserState>) -> Result<Vec<WorkspaceItem>, String> {
    let db_guard = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Workspace database is not ready".to_string())?;
    workspace_repo::list_workspaces(conn)
}

#[tauri::command]
pub async fn get_active_workspace(state: State<'_, BrowserState>) -> Result<String, String> {
    Ok(state
        .session()
        .edit_active_workspace_id()
        .lock()
        .map(|id| id.clone())
        .unwrap_or_else(|_| "ws-personal".to_string()))
}

/// Reports the durable per-workspace content isolation preference.
#[tauri::command]
pub fn get_workspace_isolation(state: State<'_, BrowserState>) -> bool {
    state.policies().workspace_isolation()
}

/// Applies and persists per-workspace content isolation. Enabling or disabling
/// the setting reconciles profile bindings after persistence. Only documents
/// whose effective data folder changes lose their live state.
#[tauri::command]
pub async fn set_workspace_isolation(
    app: AppHandle,
    state: State<'_, BrowserState>,
    enabled: bool,
) -> Result<(), String> {
    let _operation = state.session().operation().await;
    let previous = state.policies().workspace_isolation();
    if previous == enabled {
        return Ok(());
    }
    {
        let db_guard = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn = db_guard
            .as_ref()
            .ok_or_else(|| "Preference database is not ready".to_string())?;
        preferences_repo::store(
            conn,
            preferences_repo::WORKSPACE_ISOLATION_KEY,
            if enabled { "true" } else { "false" },
        )
        .map_err(|e| format!("Failed to persist workspace isolation: {e}"))?;
    }
    state.policies().set_workspace_isolation(enabled);
    let result = crate::window::webview_manager::sync_all_webviews(&app, &state);
    if let Err(error) = result {
        let _ = crate::security::emit_shell(
            &app,
            "browser-error",
            format!("The isolation setting was saved, but pages could not be rebuilt: {error}"),
        );
    }
    let _ = crate::security::emit_shell(
        &app,
        "browser-error",
        "Pages were reloaded to apply per-workspace isolation",
    );
    let _ = crate::security::emit_shell(&app, "tabs-updated", state.snapshot_packet());
    Ok(())
}

#[tauri::command]
pub async fn switch_workspace(
    app: AppHandle,
    state: State<'_, BrowserState>,
    workspace_id: String,
) -> Result<TabSnapshot, String> {
    let _operation = state.session().operation().await;
    let exists = state
        .session()
        .edit_workspaces()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .any(|workspace| workspace.id == workspace_id);
    if !exists {
        return Err("Workspace not found".to_string());
    }
    let existing_tab_id = state
        .session()
        .edit_tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .find(|tab| tab.workspace_id == workspace_id)
        .map(|tab| tab.id.clone());
    let existing_tab_id = state
        .session()
        .edit_split_layouts()
        .lock()
        .map_err(|e| e.to_string())?
        .get(&workspace_id)
        .map(|layout| layout.focused_tab_id.clone())
        .or(existing_tab_id);
    if let Some(tab_id) = existing_tab_id {
        return switch_tab_inner(&app, &state, tab_id);
    }
    let before = crate::state::tab_checkpoint::TabCheckpoint::capture(&state)?;
    *state
        .session()
        .edit_active_workspace_id()
        .lock()
        .map_err(|e| e.to_string())? = workspace_id;

    let result = create_tab_inner(&app, &state, Some(DEFAULT_HOME_URL.to_string()), None, None);
    if result.is_err() {
        before.restore(&state)?;
    }
    result
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
    let _operation = state.session().operation().await;
    let workspace = WorkspaceItem {
        id,
        name,
        icon,
        accent_primary,
        accent_secondary,
    };
    let db_guard = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Workspace database is not ready".to_string())?;
    workspace_repo::create_workspace(conn, &workspace)?;
    let list = workspace_repo::list_workspaces(conn)?;
    *state
        .session()
        .edit_workspaces()
        .lock()
        .map_err(|e| e.to_string())? = list.clone();
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
    let _operation = state.session().operation().await;
    let workspace = WorkspaceItem {
        id,
        name,
        icon,
        accent_primary,
        accent_secondary,
    };
    let db_guard = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Workspace database is not ready".to_string())?;
    workspace_repo::update_workspace(conn, &workspace)?;
    let list = workspace_repo::list_workspaces(conn)?;
    *state
        .session()
        .edit_workspaces()
        .lock()
        .map_err(|e| e.to_string())? = list.clone();
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
    let _operation = state.session().operation().await;
    if workspace_id == "ws-personal" {
        return Err("The Personal workspace cannot be deleted".to_string());
    }
    if action != "move" && action != "close" {
        return Err("Choose whether to move or close the workspace tabs".to_string());
    }

    let current_workspaces = state
        .session()
        .edit_workspaces()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
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
        .session()
        .edit_active_workspace_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let was_active = active_workspace == workspace_id;
    let previous_active_tab = state
        .session()
        .edit_active_tab_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let mut next_tabs = state
        .session()
        .edit_tabs()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let removed_ephemeral: Vec<String> = next_tabs
        .iter()
        .filter(|tab| action == "close" && tab.workspace_id == workspace_id && tab.is_ephemeral)
        .map(|tab| tab.id.clone())
        .collect();
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
    let mut next_layouts = state
        .session()
        .edit_split_layouts()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    next_layouts.retain(|workspace, layout| {
        layout.normalize(&next_tabs);
        workspace != &workspace_id && layout.valid(&next_tabs)
    });
    let mut next_active_tab = if was_active && action == "close" {
        next_layouts
            .get(&next_active_workspace)
            .map(|layout| layout.focused_tab_id.clone())
            .or_else(|| {
                next_tabs
                    .iter()
                    .rev()
                    .find(|tab| tab.workspace_id == next_active_workspace)
                    .map(|tab| tab.id.clone())
            })
            .unwrap_or_default()
    } else if next_tabs.iter().any(|tab| {
        tab.id == previous_active_tab && (!was_active || tab.workspace_id == next_active_workspace)
    }) {
        previous_active_tab.clone()
    } else {
        next_tabs
            .iter()
            .rev()
            .find(|tab| tab.workspace_id == next_active_workspace)
            .map(|tab| tab.id.clone())
            .unwrap_or_default()
    };

    let mut next_counter = *state
        .session()
        .edit_next_tab_counter()
        .lock()
        .map_err(|e| e.to_string())?;
    let mut created_active_tab = false;
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
        created_active_tab = true;
    }
    for tab in &mut next_tabs {
        tab.is_active = tab.id == next_active_tab;
        if tab.is_active {
            tab.is_hibernated = false;
            tab.last_active_epoch = TabInfo::current_epoch_secs();
        }
    }
    let active_url = next_tabs
        .iter()
        .find(|tab| tab.id == next_active_tab)
        .map(|tab| tab.url.clone())
        .ok_or_else(|| "Could not select an active tab".to_string())?;

    if let Some(layout) = next_layouts.get_mut(&next_active_workspace) {
        layout.focus_tab(&next_active_tab);
    }
    for tab in &mut next_tabs {
        tab.is_visible = tab.is_active
            || next_layouts
                .get(&next_active_workspace)
                .is_some_and(|l| l.contains(&tab.id));
        if tab.is_visible {
            tab.is_hibernated = false;
        }
    }
    let should_record_activation = was_active && previous_active_tab != next_active_tab;

    let (updated_workspaces, mut committed_tabs) = {
        let mut db_guard = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let conn: &mut Connection = db_guard
            .as_mut()
            .ok_or_else(|| "Workspace database is not ready".to_string())?;
        let transaction = conn.transaction().map_err(|e| e.to_string())?;
        save_session_tabs(&transaction, &next_tabs)?;
        split_repo::save(&transaction, &next_layouts)?;
        workspace_repo::delete_workspace(&transaction, &workspace_id)?;
        if should_record_activation {
            if created_active_tab {
                tab_usage_repo::reset_tab_history(&transaction, &next_active_tab)?;
            }
            tab_usage_repo::record_activation(
                &transaction,
                &next_active_tab,
                TabInfo::current_epoch_secs(),
            )?;
        }
        let list = workspace_repo::list_workspaces(&transaction)?;
        transaction.commit().map_err(|e| e.to_string())?;
        (list, next_tabs)
    };

    *state
        .session()
        .edit_workspaces()
        .lock()
        .map_err(|e| e.to_string())? = updated_workspaces.clone();
    {
        let mut live = state
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?;
        // Keep document events received while the transaction was waiting for SQLite.
        for tab in &mut committed_tabs {
            if let Some(current) = live.iter().find(|current| current.id == tab.id) {
                tab.url.clone_from(&current.url);
                tab.title.clone_from(&current.title);
                tab.navigation.clone_from(&current.navigation);
                tab.activity.clone_from(&current.activity);
            }
        }
        *live = committed_tabs;
    }
    *state
        .session()
        .edit_split_layouts()
        .lock()
        .map_err(|e| e.to_string())? = next_layouts;
    *state
        .session()
        .edit_split_preview()
        .lock()
        .map_err(|e| e.to_string())? = None;
    *state
        .session()
        .edit_active_workspace_id()
        .lock()
        .map_err(|e| e.to_string())? = next_active_workspace.clone();
    *state
        .session()
        .edit_active_tab_id()
        .lock()
        .map_err(|e| e.to_string())? = next_active_tab;
    *state
        .session()
        .edit_next_tab_counter()
        .lock()
        .map_err(|e| e.to_string())? = next_counter;

    if was_active {
        let _ = crate::security::emit_shell(&app, "content-url-changed", &active_url);
    }
    for id in removed_ephemeral {
        if let Err(error) = crate::window::webview_manager::close_tab_webview(&app, &id) {
            let _ = crate::security::emit_shell(&app, "browser-error", error);
        } else {
            crate::window::ephemeral::purge_ephemeral_tab_data(&app, &id);
        }
    }
    reconcile_committed(&app, &state);
    // The workspace no longer exists; reclaim its isolated data folder when
    // isolation is on. Live document teardown finished during reconciliation.
    crate::window::workspace_data::remove(&app, &workspace_id);
    let tabs = state.snapshot_packet();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &tabs);

    Ok(WorkspaceDeleteResult {
        workspaces: updated_workspaces,
        tabs,
        active_workspace_id: next_active_workspace,
    })
}
