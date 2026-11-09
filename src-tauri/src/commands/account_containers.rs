use crate::database::account_containers_repo::{self as repo, AccountContainer};
use crate::state::{
    browser_state::{BrowserState, TabSnapshot},
    tab_checkpoint::TabCheckpoint,
};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn get_account_containers(
    state: State<'_, BrowserState>,
) -> Result<Vec<AccountContainer>, String> {
    let _op = state.tab_operations.lock().await;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    repo::list(db.as_ref().ok_or("Database unavailable")?)
}

#[tauri::command]
pub async fn create_account_container(
    state: State<'_, BrowserState>,
    name: String,
) -> Result<Vec<AccountContainer>, String> {
    let _op = state.tab_operations.lock().await;
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|_| "Random source unavailable")?;
    let id: String = random.iter().map(|b| format!("{b:02x}")).collect();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Database unavailable")?;
    repo::insert(conn, &id, &name)?;
    repo::list(conn)
}

#[tauri::command]
pub async fn rename_account_container(
    state: State<'_, BrowserState>,
    id: String,
    name: String,
) -> Result<Vec<AccountContainer>, String> {
    let _op = state.tab_operations.lock().await;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Database unavailable")?;
    repo::rename(conn, &id, &name)?;
    repo::list(conn)
}

#[tauri::command]
pub async fn create_container_tab(
    app: AppHandle,
    state: State<'_, BrowserState>,
    id: String,
) -> Result<TabSnapshot, String> {
    let _op = state.tab_operations.lock().await;
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if !repo::valid_id(&id)
            || !repo::list(db.as_ref().ok_or("Database unavailable")?)?
                .iter()
                .any(|item| item.id == id && !item.deleting)
        {
            return Err("Container unavailable".into());
        }
    }
    super::tabs::create_tab_inner_ext(
        &app,
        &state,
        None,
        None,
        None,
        super::tabs::TabProfile {
            container_id: Some(id),
            ..Default::default()
        },
    )
}

#[tauri::command]
pub async fn set_tab_account_container(
    app: AppHandle,
    state: State<'_, BrowserState>,
    tab_id: String,
    container_id: Option<String>,
) -> Result<TabSnapshot, String> {
    let _op = state.tab_operations.lock().await;
    if let Some(id) = &container_id {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if !repo::valid_id(id)
            || !repo::list(db.as_ref().ok_or("Database unavailable")?)?
                .iter()
                .any(|item| item.id == *id && !item.deleting)
        {
            return Err("Container unavailable".into());
        }
    }
    let unchanged = {
        let tabs = state.tabs.lock().map_err(|e| e.to_string())?;
        let tab = tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .ok_or("Tab not found")?;
        if tab.is_ephemeral {
            return Err("Disposable tabs cannot use account containers".into());
        }
        tab.container_id == container_id
    };
    if unchanged {
        return Ok(state.snapshot_packet());
    }
    crate::window::webview_manager::close_tab_webview(&app, &tab_id)?;
    let result = TabCheckpoint::commit(&state, || {
        let mut tabs = state.tabs.lock().map_err(|e| e.to_string())?;
        tabs.iter_mut()
            .find(|tab| tab.id == tab_id)
            .ok_or("Tab not found")?
            .container_id = container_id;
        Ok(())
    });
    if let Err(error) = result {
        let _ = crate::window::webview_manager::sync_all_webviews(&app, &state);
        return Err(error);
    }
    crate::window::webview_manager::sync_all_webviews(&app, &state)?;
    let snapshot = state.snapshot_packet();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn delete_account_container(
    app: AppHandle,
    state: State<'_, BrowserState>,
    id: String,
) -> Result<Vec<AccountContainer>, String> {
    let _op = state.tab_operations.lock().await;
    if !repo::valid_id(&id) {
        return Err("Invalid container ID".into());
    }
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("Database unavailable")?;
        if !repo::list(conn)?.iter().any(|item| item.id == id) {
            return Err("Container not found".into());
        }
    }
    let ids: Vec<String> = state
        .tabs
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .filter(|tab| tab.container_id.as_deref() == Some(id.as_str()))
        .map(|tab| tab.id.clone())
        .collect();
    for tab_id in &ids {
        if let Err(error) = crate::window::webview_manager::close_tab_webview(&app, tab_id) {
            let _ = crate::window::webview_manager::sync_all_webviews(&app, &state);
            return Err(error);
        }
    }
    if let Err(error) = TabCheckpoint::commit(&state, || {
        for tab in state.tabs.lock().map_err(|e| e.to_string())?.iter_mut() {
            if tab.container_id.as_deref() == Some(id.as_str()) {
                tab.container_id = None;
            }
        }
        Ok(())
    }) {
        let _ = crate::window::webview_manager::sync_all_webviews(&app, &state);
        return Err(error);
    }
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("Database unavailable")?;
        conn.execute(
            "UPDATE account_containers SET deleting=1 WHERE id=?1",
            [&id],
        )
        .map_err(|e| e.to_string())?;
    }
    crate::window::webview_manager::sync_all_webviews(&app, &state)?;
    let snapshot = state.snapshot_packet();
    let _ = crate::security::emit_shell(&app, "tabs-updated", &snapshot);
    let directory = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("webview-accounts")
        .join(&id);
    if directory.exists() {
        std::fs::remove_dir_all(directory)
            .map_err(|e| format!("Container removal pending: {e}"))?;
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Database unavailable")?;
    conn.execute(
        "DELETE FROM account_containers WHERE id=?1 AND deleting=1",
        [&id],
    )
    .map_err(|e| e.to_string())?;
    repo::list(conn)
}
