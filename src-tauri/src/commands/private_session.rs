use crate::private_session::{self, PrivateRole};
use tauri::{AppHandle, State};

/// Opens a separate private window with its own disposable profile.
#[tauri::command]
pub fn open_private_window(app: AppHandle, role: State<'_, PrivateRole>) -> Result<(), String> {
    private_session::open_window(&app, &role)
}
