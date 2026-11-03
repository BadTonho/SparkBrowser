use tauri::AppHandle;

#[tauri::command]
pub fn set_new_tab_preferences(
    app: AppHandle,
    engine: String,
    language: String,
) -> Result<(), String> {
    crate::window::new_tab::set_preferences(&app, engine, language)
}
