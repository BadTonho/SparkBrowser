//! Site icons on Linux: WebKitGTK reports the page icon as a cairo surface, which is stored as a
//! small PNG exactly like the Windows capture does.
use super::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use webkit2gtk::{LoadEvent, WebContextExt, WebViewExt};

/// Icons larger than this are scaled down so they fit the stored-icon size limit.
const MAX_SIDE: i32 = 64;

pub fn install(
    app: &AppHandle,
    view: &tauri::Webview,
    tab_id: &str,
    token: &str,
) -> Result<(), String> {
    let directory = super::super::workspace_data::content_directory(app, tab_id)?
        .or_else(|| app.path().app_cache_dir().ok())
        .map(|dir| dir.join("favicons"));
    let (app, label, token) = (app.clone(), view.label().to_owned(), token.to_owned());
    view.with_webview(move |platform| {
        let web_view = platform.inner();
        // WebKit only reports icons while its favicon database is enabled for the context.
        if let (Some(context), Some(directory)) = (web_view.context(), directory) {
            if std::fs::create_dir_all(&directory).is_ok() {
                if let Some(path) = directory.to_str() {
                    context.set_favicon_database_directory(Some(path));
                }
            }
        }
        // The history epoch is fixed when a document starts loading, so an icon that arrives after
        // the history was cleared is recognised as belonging to the cleared history.
        let document_epoch = Arc::new(AtomicU64::new(super::epoch(&app)));
        let (a, l, t, e) = (
            app.clone(),
            label.clone(),
            token.clone(),
            document_epoch.clone(),
        );
        web_view.connect_favicon_notify(move |view| {
            capture(&a, &l, &t, view, e.load(Ordering::SeqCst))
        });
        web_view.connect_load_changed(move |view, event| match event {
            LoadEvent::Started => document_epoch.store(super::epoch(&app), Ordering::SeqCst),
            LoadEvent::Finished => capture(
                &app,
                &label,
                &token,
                view,
                document_epoch.load(Ordering::SeqCst),
            ),
            _ => {}
        });
    })
    .map_err(|e| e.to_string())
}

fn capture(app: &AppHandle, label: &str, token: &str, view: &webkit2gtk::WebView, epoch: u64) {
    let Some(url) = view.uri().map(|uri| uri.to_string()) else {
        return;
    };
    let Some(key) = site_icons_repo::site_key(&url) else {
        return;
    };
    if !current(app, label, token, &url) {
        return;
    }
    let Some(bytes) = view.favicon().and_then(encode) else {
        return;
    };
    let (app, label, token) = (app.clone(), label.to_owned(), token.to_owned());
    tauri::async_runtime::spawn(async move {
        let worker_app = app.clone();
        let result = crate::database::executor::run(app.clone(), move |conn| {
            if !current(&worker_app, &label, &token, &url) {
                return Ok(false);
            }
            save(&worker_app, conn, epoch, &key, &bytes)
        })
        .await;
        if matches!(result, Ok(true)) {
            super::super::new_tab::invalidate(&app);
            crate::error_log::ignored(crate::security::emit_shell(&app, "site-icons-changed", ()));
        }
    });
}

/// Encodes the icon as a PNG no larger than `MAX_SIDE` pixels on a side.
fn encode(surface: gtk::cairo::Surface) -> Option<Vec<u8>> {
    use gtk::cairo::{Context, Format, ImageSurface};
    let source = ImageSurface::try_from(surface).ok()?;
    let (width, height) = (source.width(), source.height());
    if width <= 0 || height <= 0 {
        return None;
    }
    let scale = (MAX_SIDE as f64 / width.max(height) as f64).min(1.0);
    let target = ImageSurface::create(
        Format::ARgb32,
        ((width as f64 * scale).round() as i32).max(1),
        ((height as f64 * scale).round() as i32).max(1),
    )
    .ok()?;
    {
        let context = Context::new(&target).ok()?;
        context.scale(scale, scale);
        context.set_source_surface(&source, 0.0, 0.0).ok()?;
        context.paint().ok()?;
    }
    let mut png = Vec::new();
    target.write_to_png(&mut png).ok()?;
    (png.len() <= site_icons_repo::MAX_BYTES).then_some(png)
}
