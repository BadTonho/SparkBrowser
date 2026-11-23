use crate::{
    database::{
        executor,
        tab_dividers_repo::{self as repo, DividerEdit, TabDivider},
    },
    state::browser_state::BrowserState,
};
use tauri::{AppHandle, State};
#[tauri::command]
pub async fn get_tab_dividers(app: AppHandle) -> Result<Vec<TabDivider>, String> {
    executor::run(app, repo::load).await
}
#[tauri::command]
pub async fn set_tab_dividers(
    app: AppHandle,
    state: State<'_, BrowserState>,
    mut edit: DividerEdit,
) -> Result<Vec<TabDivider>, String> {
    let _operation = state.session().operation().await;
    {
        let tabs = state.session().tabs().lock()?;
        edit.repair_anchor(|id| tabs.iter().any(|tab| tab.id == id));
    }
    let items = executor::run(app.clone(), move |conn| repo::edit(conn, edit)).await?;
    notify(&app);
    Ok(items)
}

pub(crate) fn notify(app: &AppHandle) {
    crate::security::emit_shell_all(app, "tab-dividers-updated", ());
}
