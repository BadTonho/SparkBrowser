use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn save_qr_code_png(app: tauri::AppHandle, png_base64: String) -> Result<bool, String> {
    let bytes = STANDARD
        .decode(png_base64)
        .map_err(|_| "qr_code.invalid_png".to_string())?;
    if bytes.len() < 8 || bytes.len() > 2 * 1024 * 1024 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
    {
        return Err("qr_code.invalid_png".into());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = app
            .dialog()
            .file()
            .add_filter("PNG image", &["png"])
            .set_file_name("qr-code.png")
            .blocking_save_file()
        else {
            return Ok(false);
        };
        let path = file
            .into_path()
            .map_err(|_| "qr_code.save_failed".to_string())?;
        if path
            .extension()
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("png"))
        {
            return Err("qr_code.invalid_extension".into());
        }
        std::fs::write(path, bytes).map_err(|_| "qr_code.save_failed".to_string())?;
        Ok(true)
    })
    .await
    .map_err(|_| "qr_code.save_failed".to_string())?
}
