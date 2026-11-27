//! The small click-through window that draws the hover card next to its tab.
//!
//! The window is only as large as the card, so showing it never makes the system redraw the whole
//! browser window. Moving from one tab to the next keeps it on screen and only changes its content
//! and position. When the card closes, its content is cleared at once, the window is hidden a moment
//! later (so the next show starts from an empty frame, never from an old card) and it is destroyed
//! once the tabs have not been hovered for a while, so it does not keep a web view alive.
use super::{Anchor, Placement};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WindowBuilder,
};

const LABEL: &str = "tab-hover-card";
const DOCUMENT: &str = "tab-hover-card.html";
/// Card size in CSS pixels; `public/overlays/tab-hover-card.css` uses the same values.
const CARD_WIDTH: f64 = 280.0;
const CARD_HEIGHT: f64 = 216.0;
/// Room around the card for its shadow.
const MARGIN: f64 = 12.0;
const GAP: f64 = 6.0;
const EDGE: f64 = 8.0;
const HIDE_AFTER: Duration = Duration::from_millis(500);
const DESTROY_AFTER: Duration = Duration::from_secs(60);

/// The browser window the card window belongs to.
static OWNER: crate::sync::Mutex<String> = crate::sync::Mutex::new(String::new());
/// Changes on every show and hide, so a delayed hide or destroy knows it is out of date.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Where the card goes, in the browser window's CSS pixels: below the tab (horizontal tabs) or to
/// its right (vertical tabs), flipped and clamped to stay inside the window.
fn place(anchor: Anchor, placement: Placement, view_width: f64, view_height: f64) -> (f64, f64) {
    let (mut left, mut top);
    match placement {
        Placement::Right => {
            left = anchor.x + anchor.width + GAP;
            top = anchor.y;
            if left + CARD_WIDTH > view_width - EDGE {
                left = anchor.x - CARD_WIDTH - GAP;
            }
        }
        Placement::Below => {
            left = anchor.x;
            top = anchor.y + anchor.height + GAP;
            if top + CARD_HEIGHT > view_height - EDGE {
                top = anchor.y - CARD_HEIGHT - GAP;
            }
        }
    }
    let clamp = |value: f64, size: f64, room: f64| value.min(room - size - EDGE).max(EDGE);
    (
        clamp(left, CARD_WIDTH, view_width),
        clamp(top, CARD_HEIGHT, view_height),
    )
}

/// Clears the card now, hides its window shortly after and destroys it after a longer idle time.
pub fn hide(app: &AppHandle) {
    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let Some(view) = app.get_webview(LABEL) else {
        return;
    };
    crate::error_log::ignored(view.eval("window.hideTabHoverCard && window.hideTabHoverCard();"));
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(HIDE_AFTER).await;
        if GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }
        if let Some(window) = app.get_window(LABEL) {
            crate::error_log::ignored(visibility(&window, None));
        }
        tokio::time::sleep(DESTROY_AFTER).await;
        if GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }
        if let Some(window) = app.get_window(LABEL) {
            crate::error_log::ignored(window.destroy());
        }
    });
}

// Showing the card must not take focus or the pointer from the browser window.
#[cfg(windows)]
fn visibility(window: &tauri::Window, revision: Option<u64>) -> Result<(), String> {
    let handle = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    let app = window.app_handle().clone();
    window
        .run_on_main_thread(move || {
            if revision.is_some_and(|revision| !super::paint_is_current(&app, revision)) {
                return;
            }
            #[link(name = "user32")]
            unsafe extern "system" {
                fn ShowWindow(window: *mut std::ffi::c_void, command: i32) -> i32;
                fn IsWindowVisible(window: *mut std::ffi::c_void) -> i32;
            }
            let handle = handle as *mut std::ffi::c_void;
            let show = revision.is_some();
            // 4 = SW_SHOWNOACTIVATE, 0 = SW_HIDE. Nothing is done when it is already so.
            unsafe {
                if (IsWindowVisible(handle) != 0) != show {
                    ShowWindow(handle, if show { 4 } else { 0 });
                }
            }
        })
        .map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn visibility(window: &tauri::Window, revision: Option<u64>) -> Result<(), String> {
    let target = window.clone();
    window
        .run_on_main_thread(move || {
            if revision
                .is_some_and(|revision| !super::paint_is_current(target.app_handle(), revision))
            {
                return;
            }
            let shown = target.is_visible().unwrap_or(false);
            if revision.is_some() && !shown {
                crate::error_log::ignored(target.show());
            } else if revision.is_none() && shown {
                crate::error_log::ignored(target.hide());
            }
        })
        .map_err(|e| e.to_string())
}

fn window_size() -> LogicalSize<f64> {
    LogicalSize::new(CARD_WIDTH + 2.0 * MARGIN, CARD_HEIGHT + 2.0 * MARGIN)
}

fn create(app: &AppHandle, main: &tauri::Window, window_id: &str) -> Result<tauri::Window, String> {
    let document = app
        .get_webview(window_id)
        .ok_or("Browser shell not found")?
        .url()
        .map_err(|e| e.to_string())?
        .join(&format!("/{DOCUMENT}"))
        .map_err(|e| e.to_string())?;
    let size = window_size();
    let window = WindowBuilder::new(app, LABEL)
        .title("Spark tab preview")
        .parent(main)
        .map_err(|e| e.to_string())?
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .shadow(false)
        .skip_taskbar(true)
        .focused(false)
        .focusable(false)
        .visible(false)
        .inner_size(size.width, size.height)
        .build()
        .map_err(|e| e.to_string())?;
    let built = window
        .add_child(
            crate::window::create_webview_builder(LABEL, WebviewUrl::App(DOCUMENT.into()))
                .transparent(true)
                .focused(false)
                .on_navigation(move |url| {
                    crate::security::documents::same_document(url, &document)
                }),
            LogicalPosition::new(0.0, 0.0),
            size,
        )
        .map_err(|e| e.to_string())
        .and_then(|_| match app.get_webview(LABEL) {
            Some(view) => crate::security::harden(app, &view, true),
            None => Err("Tab preview WebView not found".into()),
        })
        .and_then(|()| {
            window
                .set_ignore_cursor_events(true)
                .map_err(|e| e.to_string())
        });
    if let Err(error) = built {
        crate::error_log::ignored(window.destroy());
        return Err(error);
    }
    Ok(window)
}

pub fn paint(
    app: &AppHandle,
    window_id: &str,
    payload: &serde_json::Value,
    placement: Placement,
    anchor: Anchor,
) -> Result<(), String> {
    let revision = payload["revision"]
        .as_u64()
        .ok_or("Missing hover card revision")?;
    if !super::paint_is_current(app, revision) {
        return Ok(());
    }
    GENERATION.fetch_add(1, Ordering::SeqCst);
    let main = app
        .get_window(window_id)
        .ok_or("Browser window not found")?;
    if main.is_minimized().map_err(|e| e.to_string())? {
        hide(app);
        return Ok(());
    }
    // The card belongs to the window it is drawn over; a card for another window is created again.
    let owned_here = OWNER.lock().is_ok_and(|owner| owner.as_str() == window_id);
    let window = match app.get_window(LABEL) {
        Some(window) if owned_here && app.get_webview(LABEL).is_some() => window,
        existing => {
            if let Some(existing) = existing {
                existing.destroy().map_err(|e| e.to_string())?;
            }
            let window = create(app, &main, window_id)?;
            if let Ok(mut owner) = OWNER.lock() {
                *owner = window_id.to_string();
            }
            window
        }
    };
    let scale = main.scale_factor().map_err(|e| e.to_string())?;
    let origin = main.inner_position().map_err(|e| e.to_string())?;
    let view = main
        .inner_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(scale);
    let (left, top) = place(anchor, placement, view.width, view.height);
    let target = PhysicalPosition::new(
        origin.x + ((left - MARGIN) * scale).round() as i32,
        origin.y + ((top - MARGIN) * scale).round() as i32,
    );
    if window.outer_position().map_err(|e| e.to_string())? != target {
        window.set_position(target).map_err(|e| e.to_string())?;
    }
    let size = window_size();
    if window.inner_size().map_err(|e| e.to_string())? != size.to_physical::<u32>(scale) {
        window.set_size(size).map_err(|e| e.to_string())?;
    }
    app.get_webview(LABEL)
        .ok_or("Tab preview WebView not found")?
        .eval(format!("(() => {{ const data={payload}; let attempts=0; const paint=()=>{{if(window.setTabHoverCard)window.setTabHoverCard(data);else if(++attempts<120)setTimeout(paint,16);}};paint(); }})();"))
        .map_err(|e| e.to_string())?;
    visibility(&window, Some(revision))
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn anchor(x: f64, y: f64, width: f64, height: f64) -> Anchor {
        Anchor {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn opens_below_horizontal_tabs_and_beside_vertical_ones() {
        assert_eq!(
            place(
                anchor(100.0, 40.0, 240.0, 28.0),
                Placement::Below,
                1200.0,
                800.0
            ),
            (100.0, 74.0)
        );
        assert_eq!(
            place(
                anchor(0.0, 200.0, 260.0, 36.0),
                Placement::Right,
                1200.0,
                800.0
            ),
            (266.0, 200.0)
        );
    }

    #[test]
    fn stays_inside_the_window() {
        let (left, _) = place(
            anchor(700.0, 40.0, 120.0, 28.0),
            Placement::Below,
            800.0,
            600.0,
        );
        assert_eq!(left, 800.0 - CARD_WIDTH - EDGE);
        let (_, top) = place(
            anchor(0.0, 560.0, 260.0, 36.0),
            Placement::Right,
            1200.0,
            600.0,
        );
        assert_eq!(top, 600.0 - CARD_HEIGHT - EDGE);
        let (_, top) = place(
            anchor(0.0, 700.0, 200.0, 28.0),
            Placement::Below,
            1200.0,
            760.0,
        );
        assert_eq!(
            top,
            700.0 - CARD_HEIGHT - GAP,
            "flips above a tab near the bottom"
        );
    }
}
