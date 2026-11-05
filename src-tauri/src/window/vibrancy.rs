use tauri::WebviewWindow;

/// Applies native OS hardware-accelerated translucent effects (Mica / Acrylic on Windows,
/// Vibrancy on macOS) to the main browser shell window.
pub fn apply_native_vibrancy(_window: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        // Prefer Mica on Windows 11; fall back to dark Acrylic if Mica is unavailable
        if window_vibrancy::apply_mica(_window, Some(true)).is_err() {
            let _ = window_vibrancy::apply_acrylic(_window, Some((14, 16, 24, 210)));
        }
    }

    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
        let _ = apply_vibrancy(_window, NSVisualEffectMaterial::HudWindow, None, None);
    }
}
