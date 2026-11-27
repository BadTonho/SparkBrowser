//! Progress and cancellation for downloads started by WebKitGTK. The engine cannot pause or resume
//! a transfer, so those stay unavailable; the downloads list shows what is received and can stop it.
use super::manager::DownloadManager;
use crate::sync::Mutex;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
use webkit2gtk::{DownloadExt, URIRequestExt, URIResponseExt, WebContextExt, WebViewExt};

/// Downloads the handler accepted that WebKit has not announced yet: (url, task id).
static EXPECTED: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
static CANCELLED: std::sync::LazyLock<Mutex<HashSet<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashSet::new()));

thread_local! {
    static CONTEXTS: RefCell<Vec<webkit2gtk::WebContext>> = const { RefCell::new(Vec::new()) };
    static ACTIVE: RefCell<HashMap<String, webkit2gtk::Download>> = RefCell::new(HashMap::new());
}

/// Remembers that the next WebKit download of `url` belongs to task `id`.
pub fn expect(url: &str, id: &str) {
    if let Ok(mut expected) = EXPECTED.lock() {
        expected.push((url.to_owned(), id.to_owned()));
    }
}

/// True once when the task was cancelled by the user rather than interrupted.
pub fn take_cancelled(id: &str) -> bool {
    CANCELLED.lock().is_ok_and(|mut ids| ids.remove(id))
}

pub fn install(app: &AppHandle, view: &tauri::Webview) -> Result<(), String> {
    let app = app.clone();
    view.with_webview(move |platform| {
        let Some(context) = platform.inner().context() else {
            return;
        };
        // Pages that share a data directory share one context; connect it once.
        if CONTEXTS.with(|known| known.borrow().contains(&context)) {
            return;
        }
        CONTEXTS.with(|known| known.borrow_mut().push(context.clone()));
        context.connect_download_started(move |_, download| track(&app, download));
    })
    .map_err(|e| e.to_string())
}

fn track(app: &AppHandle, download: &webkit2gtk::Download) {
    let Some(url) = download.request().and_then(|r| r.uri()) else {
        return;
    };
    let id = EXPECTED.lock().ok().and_then(|mut expected| {
        let index = expected
            .iter()
            .position(|(u, _)| u.as_str() == url.as_str())?;
        Some(expected.remove(index).1)
    });
    let Some(id) = id else {
        return;
    };
    ACTIVE.with(|active| active.borrow_mut().insert(id.clone(), download.clone()));
    app.state::<DownloadManager>()
        .native_capability(app, &id, true, false);
    let (progress_app, progress_id) = (app.clone(), id.clone());
    let last = RefCell::new(Instant::now() - Duration::from_secs(1));
    download.connect_received_data(move |download, _| {
        if last.borrow().elapsed() < Duration::from_millis(250) {
            return;
        }
        *last.borrow_mut() = Instant::now();
        let received = download.received_data_length();
        let total = download
            .response()
            .map(|response| response.content_length())
            .filter(|length| *length > 0);
        progress_app
            .state::<DownloadManager>()
            .update(&progress_app, &progress_id, |task| {
                task.received = received;
                task.total = total;
            });
    });
    download.connect_finished(move |_| {
        ACTIVE.with(|active| active.borrow_mut().remove(&id));
    });
}

/// Stops a running download. Returns false when this engine is not transferring `id`.
pub fn cancel(app: &AppHandle, id: &str) -> bool {
    let download = ACTIVE.with(|active| active.borrow().get(id).cloned());
    let Some(download) = download else {
        return false;
    };
    if let Ok(mut ids) = CANCELLED.lock() {
        ids.insert(id.to_owned());
    }
    download.cancel();
    let _ = app;
    true
}
