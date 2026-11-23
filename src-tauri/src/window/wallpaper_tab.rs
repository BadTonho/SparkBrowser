//! The custom background editor: a browser-owned page that opens in its own tab (`spark://wallpaper`).
//!
//! It is served from the app's own assets like the settings tab, but has no privileged bridge: the page
//! cannot call any command.
use tauri::{AppHandle, Url};

pub const URL: &str = "spark://wallpaper";

pub fn is_wallpaper_url(raw: &str) -> bool {
    raw == URL
}

pub fn document_url(app: &AppHandle) -> Result<Url, String> {
    let mut url = crate::security::documents::asset_base(app)
        .join("/index.html")
        .map_err(|e| e.to_string())?;
    url.set_query(Some("surface=wallpaper-editor"));
    Ok(url)
}

pub fn is_document(app: &AppHandle, url: &Url) -> bool {
    document_url(app)
        .is_ok_and(|expected| crate::security::documents::same_document(url, &expected))
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_the_exact_internal_address_is_the_editor() {
        assert!(super::is_wallpaper_url("spark://wallpaper"));
        for raw in [
            "spark://wallpaper/",
            "spark://wallpaper?x=1",
            "spark://newtab",
            "https://wallpaper",
        ] {
            assert!(!super::is_wallpaper_url(raw), "{raw}");
        }
    }
}
