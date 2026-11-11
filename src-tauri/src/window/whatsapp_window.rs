//! WhatsApp Web hosted as a child WebView anchored under the titlebar button (Windows).
//! The child has no IPC grants and shares the default WebView2 profile, so the login
//! survives restarts. It is hidden rather than destroyed, so the connection stays alive.
use crate::commands::whatsapp::Anchor;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    webview::NewWindowResponse, AppHandle, LogicalPosition, LogicalSize, Manager, Rect, Url,
    WebviewUrl,
};

pub const LABEL: &str = "whatsapp-popover-webview";
const START_URL: &str = "https://web.whatsapp.com/";
// WhatsApp Web lays out for a wide viewport; a narrow popover clips its content.
const WIDTH: f64 = 720.0;
const HEIGHT: f64 = 680.0;
const GAP: f64 = 6.0;
const MARGIN: f64 = 8.0;

static OPEN: AtomicBool = AtomicBool::new(false);

pub fn toggle(app: &AppHandle, anchor: Anchor) -> Result<(), String> {
    if OPEN.load(Ordering::Acquire) {
        return hide(app);
    }
    show(app, anchor)
}

fn show(app: &AppHandle, anchor: Anchor) -> Result<(), String> {
    let main = app.get_window("main").ok_or("Main window not found")?;
    let scale = main.scale_factor().map_err(|error| error.to_string())?;
    let window = main
        .inner_size()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(scale);
    let (width, height) = size(window.width, window.height);
    let (x, y) = position(anchor, width, height, window.width, window.height);

    let view = match app.get_webview(LABEL) {
        Some(view) => view,
        None => create(app, &main, x, y, width, height)?,
    };
    view.set_bounds(Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(width, height).into(),
    })
    .map_err(|error| error.to_string())?;
    view.show().map_err(|error| error.to_string())?;
    view.set_focus().map_err(|error| error.to_string())?;
    OPEN.store(true, Ordering::Release);
    Ok(())
}

fn create(
    app: &AppHandle,
    main: &tauri::Window,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<tauri::Webview, String> {
    let url = Url::parse(START_URL).map_err(|error| error.to_string())?;
    let view = main
        .add_child(
            crate::window::create_webview_builder(LABEL, WebviewUrl::External(url))
                .on_new_window(|_, _| NewWindowResponse::Deny),
            LogicalPosition::new(x, y),
            LogicalSize::new(width, height),
        )
        .map_err(|error| error.to_string())?;
    if let Err(error) = crate::security::harden(app, &view, false) {
        let _ = view.close();
        return Err(error);
    }
    Ok(view)
}

pub fn hide(app: &AppHandle) -> Result<(), String> {
    OPEN.store(false, Ordering::Release);
    if let Some(view) = app.get_webview(LABEL) {
        view.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Preferred popover size, shrunk only when the browser window is too small to hold it.
fn size(window_width: f64, window_height: f64) -> (f64, f64) {
    (
        WIDTH.min(window_width - 2.0 * MARGIN).max(MARGIN),
        HEIGHT.min(window_height - 2.0 * MARGIN).max(MARGIN),
    )
}

/// Right-aligns the popover with the button, below it when there is room, otherwise above.
fn position(
    anchor: Anchor,
    width: f64,
    height: f64,
    window_width: f64,
    window_height: f64,
) -> (f64, f64) {
    let max_x = (window_width - width - MARGIN).max(MARGIN);
    let x = (anchor.left + anchor.width - width).clamp(MARGIN, max_x);
    let below = anchor.top + anchor.height + GAP;
    let y = if below + height + MARGIN <= window_height {
        below
    } else {
        (anchor.top - GAP - height).max(MARGIN)
    };
    (x, y)
}
