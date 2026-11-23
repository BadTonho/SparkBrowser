use crate::state::{
    browser_state::{BrowserState, TabSnapshot},
    tab_model::TabInfo,
};
use serde::Serialize;
use tauri::{AppHandle, State};
#[derive(Serialize)]
pub struct PreservedTab {
    pub id: String,
    pub reason: String,
}
#[derive(Serialize)]
pub struct BulkCloseResult {
    pub snapshot: TabSnapshot,
    pub closed: Vec<String>,
    pub preserved: Vec<PreservedTab>,
}
pub fn close_candidates(tabs: &[TabInfo], ids: &[String]) -> (Vec<String>, Vec<PreservedTab>) {
    let mut closed = Vec::new();
    let mut preserved = Vec::new();
    for id in ids.iter().collect::<std::collections::BTreeSet<_>>() {
        match tabs.iter().find(|t| &t.id == id) {
            Some(tab) if !tab.protected_from_hibernation() => closed.push(id.clone()),
            Some(_) => preserved.push(PreservedTab {
                id: id.clone(),
                reason: "Active, pinned, loading, edited form, media or download".into(),
            }),
            None => preserved.push(PreservedTab {
                id: id.clone(),
                reason: "Tab no longer exists".into(),
            }),
        }
    }
    (closed, preserved)
}
#[tauri::command]
pub async fn close_tabs_reviewed(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<'_, BrowserState>,
    tab_ids: Vec<String>,
) -> Result<BulkCloseResult, String> {
    let _operation = state.session().operation().await;
    let before = crate::state::tab_checkpoint::TabCheckpoint::capture(&state)?;
    let previous = &before.tabs;
    let (closed, preserved) = {
        let mut tabs = state
            .session()
            .edit_tabs()
            .lock()
            .map_err(|e| e.to_string())?;
        let result = close_candidates(&tabs, &tab_ids);
        tabs.retain(|t| !result.0.contains(&t.id));
        result
    };
    if !closed.is_empty() {
        let rows = crate::database::closed_tabs_repo::records(
            previous,
            &closed,
            &state
                .session()
                .edit_workspaces()
                .lock()
                .map_err(|e| e.to_string())?,
            crate::state::tab_model::TabInfo::current_epoch_secs() as i64,
        );
        state
            .session()
            .edit_split_layouts()
            .lock()
            .map_err(|e| e.to_string())?
            .retain(|_, layout| closed.iter().all(|id| layout.remove_member(id)));
        if let Err(error) = state.persist_tab_closure(&rows, previous, &closed) {
            before.restore(&state)?;
            return Err(error);
        }
        crate::commands::tab_dividers::notify(&app);
    }
    for tab in before
        .tabs
        .iter()
        .filter(|tab| tab.is_ephemeral && closed.contains(&tab.id))
    {
        if let Err(error) = crate::window::webview_manager::close_tab_webview(&app, &tab.id) {
            crate::error_log::ignored(crate::security::emit_shell(&app, "browser-error", error));
        } else {
            crate::window::ephemeral::purge_ephemeral_tab_data(&app, &tab.id);
        }
    }
    let snapshot = super::closed_tabs::publish(
        &app,
        &state,
        &crate::commands::access::caller_window(&webview),
    );
    Ok(BulkCloseResult {
        snapshot,
        closed,
        preserved,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn batch_closing_revalidates_protection_and_deduplicates_ids() {
        let mut tabs: Vec<_> = (0..7)
            .map(|i| TabInfo::new_default(format!("t{i}"), "https://example.test/".into(), i == 0))
            .collect();
        tabs[1].is_pinned = true;
        tabs[2].activity.dirty_form = true;
        tabs[3].activity.playing_media = true;
        tabs[4].activity.downloading = true;
        tabs[5].navigation.loading = true;
        let mut ids: Vec<_> = tabs.iter().map(|t| t.id.clone()).collect();
        ids.push("t6".into());
        ids.push("missing".into());
        let (closed, preserved) = close_candidates(&tabs, &ids);
        assert_eq!(closed, vec!["t6"]);
        assert_eq!(preserved.len(), 7);
    }
}
