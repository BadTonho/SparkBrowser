//! WhatsApp Web hosted as a child WebView anchored under the titlebar button (Windows).
//! The child has no IPC grants and shares the default WebView2 profile, so the login
//! survives restarts. It is hidden rather than destroyed, so the connection stays alive.
use crate::commands::whatsapp::{Anchor, DisplayMode};
use crate::state::browser_state::ViewportBounds;
use crate::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use tauri::{
    webview::NewWindowResponse, AppHandle, LogicalPosition, LogicalSize, Manager, Rect, Url,
    WebviewUrl,
};

pub const LABEL: &str = "whatsapp-popover-webview";
const START_URL: &str = "https://web.whatsapp.com/";
const WIDTH: f64 = 560.0;
const HEIGHT: f64 = 680.0;
const SIDE_PANEL_WIDTH: f64 = 420.0;
const MIN_PAGE_WIDTH: f64 = 100.0;
const WINDOW_SPACE_FOR_PANEL: f64 = 220.0;
const GAP: f64 = 6.0;
const SIDE_PANEL_GAP: f64 = 8.0;
const MARGIN: f64 = 8.0;
const MODE_POPOVER: u8 = 0;
const MODE_SIDE_PANEL: u8 = 1;

static OPEN: AtomicBool = AtomicBool::new(false);
static ACTIVE_MODE: AtomicU8 = AtomicU8::new(MODE_POPOVER);
static LAST_ANCHOR: Mutex<Option<Anchor>> = Mutex::new(None);

pub fn toggle(app: &AppHandle, anchor: Anchor, mode: DisplayMode) -> Result<(), String> {
    if OPEN.load(Ordering::Acquire) {
        return hide(app);
    }
    show(app, anchor, mode)
}

fn show(app: &AppHandle, anchor: Anchor, mode: DisplayMode) -> Result<(), String> {
    let main = app.get_window("main").ok_or("Main window not found")?;
    let scale = main.scale_factor().map_err(|error| error.to_string())?;
    let window = main
        .inner_size()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(scale);
    let (x, y, width, height) = match mode {
        DisplayMode::Popover => {
            let (width, height) = size(window.width, window.height);
            let (x, y) = position(anchor, width, height, window.width, window.height);
            (x, y, width, height)
        }
        DisplayMode::SidePanel => side_panel_from_anchor(anchor, window.width, window.height),
    };

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
    *LAST_ANCHOR.lock().map_err(|error| error.to_string())? = Some(anchor);
    ACTIVE_MODE.store(mode_code(mode), Ordering::Release);
    OPEN.store(true, Ordering::Release);
    Ok(())
}

/// Keeps the active child aligned with the visible browser viewport while the window changes.
pub fn resize(app: &AppHandle, viewport: ViewportBounds) -> Result<(), String> {
    if !OPEN.load(Ordering::Acquire) {
        return Ok(());
    }
    let Some(view) = app.get_webview(LABEL) else {
        OPEN.store(false, Ordering::Release);
        return Ok(());
    };
    let main = app.get_window("main").ok_or("Main window not found")?;
    let scale = main.scale_factor().map_err(|error| error.to_string())?;
    let window = main
        .inner_size()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(scale);
    let rect = if ACTIVE_MODE.load(Ordering::Acquire) == MODE_SIDE_PANEL {
        side_panel_from_viewport(viewport, window.width, window.height)
    } else {
        let anchor = LAST_ANCHOR
            .lock()
            .map_err(|error| error.to_string())?
            .unwrap_or(Anchor {
                left: 0.0,
                top: 0.0,
                width: 0.0,
                height: 0.0,
            });
        let (width, height) = size(window.width, window.height);
        let (x, y) = position(anchor, width, height, window.width, window.height);
        (x, y, width, height)
    };
    set_bounds(&view, rect)
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
        crate::error_log::ignored(view.close());
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

fn set_bounds(
    view: &tauri::Webview,
    (x, y, width, height): (f64, f64, f64, f64),
) -> Result<(), String> {
    view.set_bounds(Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(width, height).into(),
    })
    .map_err(|error| error.to_string())
}

fn mode_code(mode: DisplayMode) -> u8 {
    match mode {
        DisplayMode::Popover => MODE_POPOVER,
        DisplayMode::SidePanel => MODE_SIDE_PANEL,
    }
}

fn side_panel_width(window_width: f64) -> f64 {
    SIDE_PANEL_WIDTH.min((window_width - WINDOW_SPACE_FOR_PANEL).max(MIN_PAGE_WIDTH))
}

fn side_panel_from_anchor(
    anchor: Anchor,
    window_width: f64,
    window_height: f64,
) -> (f64, f64, f64, f64) {
    let width = side_panel_width(window_width);
    let x = (window_width - width - MARGIN).max(MARGIN);
    let y = (anchor.top + anchor.height + GAP).clamp(MARGIN, (window_height - MARGIN).max(MARGIN));
    let height = (window_height - y - MARGIN).max(MARGIN);
    (x, y, width, height)
}

fn side_panel_from_viewport(
    viewport: ViewportBounds,
    window_width: f64,
    window_height: f64,
) -> (f64, f64, f64, f64) {
    let width = side_panel_width(window_width);
    let x = (viewport.x + viewport.width + SIDE_PANEL_GAP)
        .min(window_width - MARGIN)
        .max(MARGIN);
    let actual_width = (window_width - x - MARGIN).max(MARGIN).min(width);
    let y = viewport.y.max(MARGIN);
    let height = (window_height - y - MARGIN)
        .max(MARGIN)
        .min(viewport.height.max(MARGIN));
    (x, y, actual_width, height)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_popover_uses_reference_size_and_shrinks_to_window() {
        assert_eq!(size(1200.0, 900.0), (560.0, 680.0));
        assert_eq!(size(400.0, 500.0), (384.0, 484.0));
    }

    #[test]
    fn side_panel_matches_the_reserved_column_and_keeps_page_space() {
        let viewport = ViewportBounds {
            x: 8.0,
            y: 80.0,
            width: 556.0,
            height: 700.0,
        };
        assert_eq!(side_panel_width(1000.0), 420.0);
        assert_eq!(
            side_panel_from_viewport(viewport, 1000.0, 800.0),
            (572.0, 80.0, 420.0, 700.0)
        );
        assert_eq!(side_panel_width(600.0), 380.0);
    }
}
