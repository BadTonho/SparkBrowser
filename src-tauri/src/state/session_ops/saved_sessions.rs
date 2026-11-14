use crate::database::saved_sessions_repo::{self as repo, SavedSession};
use crate::state::tab_checkpoint::TabCheckpoint;
use crate::state::{
    browser_state::{BrowserState, TabSnapshot},
    tab_model::TabInfo,
};
use serde::Serialize;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn get_saved_sessions(
    state: State<'_, BrowserState>,
) -> Result<Vec<SavedSession>, String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    repo::list(db.as_ref().ok_or("Database is not ready")?)
}

#[tauri::command]
pub async fn save_named_session(
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    id: Option<i64>,
    name: String,
) -> Result<(), String> {
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
    let tabs: Vec<_> = state
        .snapshot_tabs()
        .into_iter()
        .filter(|tab| tab.window_id == window_id && tab.workspace_id == workspace)
        .collect();
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    repo::save(
        db.as_ref().ok_or("Database is not ready")?,
        id,
        &name,
        &tabs,
    )
}

#[tauri::command]
pub async fn rename_named_session(
    state: State<'_, BrowserState>,
    id: i64,
    name: String,
) -> Result<(), String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    repo::rename(db.as_ref().ok_or("Database is not ready")?, id, &name)
}

#[tauri::command]
pub async fn delete_named_session(state: State<'_, BrowserState>, id: i64) -> Result<(), String> {
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    repo::delete(db.as_ref().ok_or("Database is not ready")?, id)
}

#[derive(Serialize)]
pub struct SessionRestore {
    pub snapshot: TabSnapshot,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub async fn restore_named_session(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    id: i64,
    expected_session: Option<SavedSession>,
) -> Result<SessionRestore, String> {
    let window_id = crate::commands::access::caller_window(&webview);
    let window = state
        .session()
        .window(&window_id)
        .ok_or("windows.not_found")?;
    let _operation = state.session().operation().await;
    let pages = {
        let db = state
            .infrastructure()
            .db()
            .lock()
            .map_err(|e| e.to_string())?;
        let current = repo::get(db.as_ref().ok_or("Database is not ready")?, id)?;
        if expected_session.as_ref().is_some_and(|s| *s != current) {
            return Err("command_search.changed".into());
        }
        current.pages
    };
    let workspace = window
        .edit_active_workspace_id()
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let shield = *state
        .policies()
        .shield_enabled()
        .lock()
        .map_err(|e| e.to_string())?;
    let mut warnings = Vec::new();
    let mut selected = None;
    let mut first_allowed = None;
    let mut added = Vec::new();
    TabCheckpoint::commit(&state, || {
        {
            let mut counter = state
                .session()
                .edit_next_tab_counter()
                .lock()
                .map_err(|e| e.to_string())?;
            for page in pages {
                if !crate::window::new_tab::valid_tab_url(&page.url) {
                    return Err("Saved session contains an unsupported URL".into());
                }
                let mut tab = TabInfo::new(
                    format!("tab-{}", *counter),
                    page.url,
                    workspace.clone(),
                    false,
                );
                *counter += 1;
                tab.window_id.clone_from(&window_id);
                tab.title = page.title;
                tab.is_pinned = page.is_pinned;
                tab.is_hibernated = true;
                if crate::privacy::adblock_engine::site_is_protected(&state, &tab.url)
                    && crate::privacy::adblock_engine::should_block_url(&tab.url, shield)
                {
                    warnings.push(format!("Privacy shield blocked: {}", tab.title));
                } else {
                    if first_allowed.is_none() {
                        first_allowed = Some(tab.id.clone());
                    }
                    if page.is_selected {
                        selected = Some(tab.id.clone());
                    }
                }
                added.push(tab);
            }
        }
        state
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?
            .extend(added);
        if let Some(id) = selected.or(first_allowed) {
            state.select_tab(&id)?;
        }
        Ok(())
    })?;
    super::tabs::reconcile_committed(&app, &state);
    let snapshot = crate::state::publish::publish_tabs(&app, &state, &window_id);
    Ok(SessionRestore { snapshot, warnings })
}
