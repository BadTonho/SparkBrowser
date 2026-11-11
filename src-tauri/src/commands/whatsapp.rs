use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    Popover,
    SidePanel,
}

/// Screen rectangle of the titlebar button, in CSS pixels of the browser shell.
#[derive(Clone, Copy, Deserialize)]
pub struct Anchor {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

/// Shows or hides the WhatsApp popover. Available on Windows only in this release.
#[tauri::command]
pub async fn toggle_whatsapp_popover(
    app: tauri::AppHandle,
    anchor: Anchor,
    mode: DisplayMode,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        crate::window::whatsapp_window::toggle(&app, anchor, mode)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, anchor, mode);
        Err("whatsapp.unsupported_platform".into())
    }
}
