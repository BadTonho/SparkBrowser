//! Per-site zoom on WebKitGTK: the saved factor of a site is applied when its page commits, and
//! Ctrl + mouse wheel changes it, as in other browsers. WebKit itself never zooms on its own.
use super::{SiteZoom, ZoomService};
use crate::{database::site_zoom_repo as repo, state::browser_state::BrowserState};
use gtk::{
    gdk,
    glib::Propagation,
    prelude::{WidgetExt, WidgetExtManual},
};
use tauri::{AppHandle, Manager};
use webkit2gtk::{LoadEvent, WebViewExt};

const STEPS: [f64; 17] = [
    0.25, 0.33, 0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0, 4.0, 5.0,
];

fn current(app: &AppHandle, label: &str, token: &str) -> bool {
    app.state::<BrowserState>()
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .is_ok_and(|m| m.get(label).is_some_and(|t| t == token))
}

/// The next stop of the zoom scale in the given direction.
fn step(factor: f64, up: bool) -> f64 {
    if up {
        STEPS
            .iter()
            .copied()
            .find(|s| *s > factor + 0.001)
            .unwrap_or(5.0)
    } else {
        STEPS
            .iter()
            .rev()
            .copied()
            .find(|s| *s < factor - 0.001)
            .unwrap_or(0.25)
    }
}

pub fn install(app: &AppHandle, view: &tauri::Webview, token: &str) -> Result<(), String> {
    let (app, label, token) = (app.clone(), view.label().to_owned(), token.to_owned());
    view.with_webview(move |platform| {
        let web_view = platform.inner();
        let (a, l, t) = (app.clone(), label.clone(), token.clone());
        web_view.connect_load_changed(move |view, event| {
            if event != LoadEvent::Committed || !current(&a, &l, &t) {
                return;
            }
            if let Some(uri) = view.uri() {
                view.set_zoom_level(a.state::<ZoomService>().factor(uri.as_str()));
            }
        });
        web_view.add_events(gdk::EventMask::SCROLL_MASK | gdk::EventMask::SMOOTH_SCROLL_MASK);
        web_view.connect_scroll_event(move |view, event| {
            if !event.state().contains(gdk::ModifierType::CONTROL_MASK)
                || !current(&app, &label, &token)
            {
                return Propagation::Proceed;
            }
            // Smooth scrolling reports tiny deltas; only the direction matters.
            let up = match event.direction() {
                gdk::ScrollDirection::Up => true,
                gdk::ScrollDirection::Down => false,
                gdk::ScrollDirection::Smooth => {
                    let (_, dy) = event.delta();
                    if dy == 0.0 {
                        return Propagation::Stop;
                    }
                    dy < 0.0
                }
                _ => return Propagation::Proceed,
            };
            let Some(uri) = view.uri().map(|uri| uri.to_string()) else {
                return Propagation::Proceed;
            };
            let Ok(origin) = repo::origin(&uri) else {
                return Propagation::Proceed;
            };
            let factor = step(app.state::<ZoomService>().factor(&uri), up);
            let (app, label, token) = (app.clone(), label.clone(), token.clone());
            tauri::async_runtime::spawn(async move {
                let browser = app.state::<BrowserState>();
                let _operation = browser.session().operation().await;
                if !current(&app, &label, &token) {
                    return;
                }
                crate::error_log::ignored(app
                    .state::<ZoomService>()
                    .set(&app, SiteZoom { origin, factor }));
            });
            Propagation::Stop
        });
    })
    .map_err(|e| e.to_string())
}
