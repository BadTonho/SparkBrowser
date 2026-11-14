use std::collections::HashMap;
use tauri::AppHandle;

/// Most pages a tab strip can show at once; anything beyond is ignored rather than queried.
const MAX_URLS: usize = 256;

/// Cached site icons for the given page addresses, as `data:` URLs keyed by the address as asked.
/// Pages without a cached icon are left out, so the interface keeps its generic glyph for them.
#[tauri::command]
pub async fn site_icons(
    app: AppHandle,
    urls: Vec<String>,
) -> Result<HashMap<String, String>, String> {
    let urls: Vec<String> = urls.into_iter().take(MAX_URLS).collect();
    crate::database::executor::run(app, move |conn| {
        let mut icons = HashMap::new();
        for url in urls {
            let Some(key) = crate::database::site_icons_repo::site_key(&url) else {
                continue;
            };
            if let Some(icon) = crate::database::site_icons_repo::data_url(conn, &key) {
                icons.insert(url, icon);
            }
        }
        Ok(icons)
    })
    .await
}
