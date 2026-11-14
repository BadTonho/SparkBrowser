use crate::private_session::{self, PrivateRole};
use tauri::{AppHandle, State};

/// Whether this process is a private window, so the interface can say so instead of looking normal.
#[tauri::command]
pub fn get_private_window_state(role: State<'_, PrivateRole>) -> bool {
    role.is_private()
}

/// Opens a separate private window with its own disposable profile.
#[tauri::command]
pub fn open_private_window(app: AppHandle, role: State<'_, PrivateRole>) -> Result<(), String> {
    private_session::open_window(&app, &role)
}
