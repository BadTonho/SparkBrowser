//! Hover card of a tab: a small card next to the tab under the pointer with its title, a picture of
//! the page and the memory it uses. Pages cover the browser shell, so the card is drawn by its own
//! click-through window above the browser window, like the tab drag preview.
//!
//! Pictures are taken only of pages that are on screen (a hidden page has nothing to draw). Each
//! tab keeps its latest picture in memory, so a tab in the background or asleep still shows how it
//! looked the last time it was visible. Pictures are never written to disk.
use crate::state::{browser_state::BrowserState, tab_model::webview_label_for_tab};
use crate::sync::Mutex;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

mod capture;
mod surface;
pub use surface::hide as hide_surface;

/// Pictures kept at most; the least recently used ones are dropped first.
const MAX_PICTURES: usize = 40;
/// A page on screen is not pictured again sooner than this.
const RECAPTURE_AFTER: Duration = Duration::from_millis(1500);

#[derive(Default)]
pub struct HoverCard(Mutex<CardState>);

#[derive(Default)]
struct CardState {
    revision: u64,
    card: Option<(String, Card)>,
    pictures: HashMap<String, Picture>,
    /// Tabs being pictured right now, so overlapping requests do not picture them twice.
    capturing: Vec<String>,
    clock: u64,
}

struct Picture {
    data_url: String,
    taken: Instant,
    used: u64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    tab_id: String,
    title: String,
    memory: String,
    no_preview: String,
    locale: String,
    placement: Placement,
    anchor: Anchor,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Placement {
    Below,
    Right,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct Anchor {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Anchor {
    fn is_valid(&self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|value| value.is_finite())
            && self.width >= 0.0
            && self.height >= 0.0
    }
}

fn service(app: &AppHandle) -> tauri::State<'_, HoverCard> {
    app.state::<HoverCard>()
}

pub(super) fn paint_is_current(app: &AppHandle, revision: u64) -> bool {
    app.try_state::<HoverCard>().is_some_and(|service| {
        service
            .0
            .lock()
            .is_ok_and(|card| card.card.is_some() && card.revision == revision)
    })
}

/// Hides the card and forgets it, for example when a panel opens or a drag starts.
pub fn dismiss(app: &AppHandle) {
    if let Some(service) = app.try_state::<HoverCard>() {
        if let Ok(mut card) = service.0.lock() {
            card.card = None;
        }
    }
    hide_surface(app);
}

fn suppressed(app: &AppHandle, state: &BrowserState) -> bool {
    super::tab_drag_preview::blocked(state)
        || super::tab_drag_preview::active(app)
        || app.state::<crate::shutdown::ShutdownService>().blocked()
}

/// Tabs of `window_id` whose pages are on screen and can be pictured.
fn visible_tabs(app: &AppHandle, state: &BrowserState, window_id: &str) -> Vec<String> {
    let Ok(panes) = super::viewport::visible_bounds_in(state, window_id) else {
        return Vec::new();
    };
    if app
        .get_window(window_id)
        .is_none_or(|window| window.is_minimized().unwrap_or(true))
    {
        return Vec::new();
    }
    let tabs = state.snapshot_tabs();
    panes
        .into_iter()
        .map(|(id, _)| id)
        .filter(|id| tabs.iter().any(|tab| &tab.id == id && !tab.is_hibernated))
        .filter(|id| app.get_webview(&webview_label_for_tab(id)).is_some())
        .collect()
}

fn picture_of(app: &AppHandle, tab_id: &str) -> Option<String> {
    let service = service(app);
    let mut card = service.0.lock().ok()?;
    card.clock += 1;
    let clock = card.clock;
    let picture = card.pictures.get_mut(tab_id)?;
    picture.used = clock;
    Some(picture.data_url.clone())
}

/// Claims the tabs that need a new picture; the caller must release them with `store`.
fn claim(app: &AppHandle, ids: Vec<String>) -> Vec<String> {
    let service = service(app);
    let Ok(mut card) = service.0.lock() else {
        return Vec::new();
    };
    let wanted: Vec<String> = ids
        .into_iter()
        .filter(|id| !card.capturing.contains(id))
        .filter(|id| {
            card.pictures
                .get(id)
                .is_none_or(|picture| picture.taken.elapsed() >= RECAPTURE_AFTER)
        })
        .collect();
    card.capturing.extend(wanted.iter().cloned());
    wanted
}

fn store(app: &AppHandle, state: &BrowserState, tab_id: &str, data_url: Option<String>) {
    let service = service(app);
    let Ok(mut card) = service.0.lock() else {
        return;
    };
    card.capturing.retain(|id| id != tab_id);
    let Some(data_url) = data_url else {
        return;
    };
    card.clock += 1;
    let used = card.clock;
    card.pictures.insert(
        tab_id.to_string(),
        Picture {
            data_url,
            taken: Instant::now(),
            used,
        },
    );
    // Closed tabs lose their picture; past the limit, the least recently used go first.
    let open: Vec<String> = state
        .snapshot_tabs()
        .into_iter()
        .map(|tab| tab.id)
        .collect();
    card.pictures.retain(|id, _| open.contains(id));
    while card.pictures.len() > MAX_PICTURES {
        let Some(oldest) = card
            .pictures
            .iter()
            .min_by_key(|(_, picture)| picture.used)
            .map(|(id, _)| id.clone())
        else {
            break;
        };
        card.pictures.remove(&oldest);
    }
}

/// Pictures the pages `window_id` shows. Returns the tabs that got a new picture.
async fn capture_visible(app: &AppHandle, window_id: &str) -> Vec<String> {
    let state = app.state::<BrowserState>();
    let ids = claim(app, visible_tabs(app, &state, window_id));
    let mut updated = Vec::new();
    for id in ids {
        let picture = match app.get_webview(&webview_label_for_tab(&id)) {
            Some(view) => capture::thumbnail(&view).await.map_err(|error| {
                crate::error_log::warn(&format!(
                    "tab_hover_card::capture_visible: could not take the hover picture of tab {id}: {error}"
                ));
            }),
            None => Err(()),
        };
        let ok = picture.is_ok();
        store(app, &state, &id, picture.ok());
        if ok {
            updated.push(id);
        }
    }
    updated
}

fn paint(app: &AppHandle, revision: u64) -> Result<(), String> {
    let Some((window_id, card)) = service(app)
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .card
        .clone()
    else {
        return Ok(());
    };
    let picture = picture_of(app, &card.tab_id);
    surface::paint(
        app,
        &window_id,
        &serde_json::json!({
            "revision": revision, "locale": card.locale, "title": card.title,
            "memory": card.memory, "noPreview": card.no_preview, "picture": picture,
        }),
        card.placement,
        card.anchor,
    )
}

/// Shows (`card` set) or hides (`None`) the hover card of a tab of the calling window. Requests
/// carry an increasing revision so a late request can never bring back a card already hidden.
#[tauri::command]
pub async fn set_tab_hover_card(
    app: AppHandle,
    webview: tauri::Webview,
    state: tauri::State<'_, BrowserState>,
    revision: u64,
    card: Option<Card>,
) -> Result<(), String> {
    let window_id = crate::commands::access::caller_window(&webview);
    {
        let service = service(&app);
        let mut latest = service.0.lock().map_err(|e| e.to_string())?;
        if revision <= latest.revision {
            return Ok(());
        }
        latest.revision = revision;
        latest.card = None;
    }
    let Some(card) = card else {
        hide_surface(&app);
        return Ok(());
    };
    if suppressed(&app, &state) {
        hide_surface(&app);
        return Ok(());
    }
    if !card.anchor.is_valid() {
        hide_surface(&app);
        return Err("Invalid hover card anchor".into());
    }
    if !state
        .snapshot_tabs()
        .iter()
        .any(|tab| tab.id == card.tab_id)
    {
        hide_surface(&app);
        return Err(format!("Hover card tab {} not found", card.tab_id));
    }
    let tab_id = card.tab_id.clone();
    service(&app).0.lock().map_err(|e| e.to_string())?.card = Some((window_id.clone(), card));
    paint(&app, revision).inspect_err(|_| hide_surface(&app))?;
    // The card is already up with the last known picture; refresh it if the page is on screen.
    if capture_visible(&app, &window_id).await.contains(&tab_id) && paint_is_current(&app, revision)
    {
        paint(&app, revision).inspect_err(|_| hide_surface(&app))?;
    }
    Ok(())
}

/// Takes new pictures of the pages the calling window shows, so a tab still has a recent picture
/// after the user switches away from it.
#[tauri::command]
pub async fn refresh_tab_previews(app: AppHandle, webview: tauri::Webview) -> Result<(), String> {
    let window_id = crate::commands::access::caller_window(&webview);
    capture_visible(&app, &window_id).await;
    Ok(())
}
