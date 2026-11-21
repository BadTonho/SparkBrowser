//! "Now playing" card for YouTube and YouTube Music tabs that are already open in the browser.
//!
//! It reads only what the browser tracks anyway (tab URL, title and playback activity). No account,
//! API key or YouTube request beyond the public thumbnail is involved, and nothing is stored.
use super::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::Read;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NowPlaying {
    pub tab_id: String,
    pub source: &'static str,
    pub video_id: String,
    pub title: String,
    pub playing: bool,
    pub position: Option<f64>,
    pub duration: Option<f64>,
    pub image: Option<String>,
}
#[derive(Default)]
struct Inner {
    /// Thumbnail of the last video seen; `None` data means the fetch failed (not retried).
    thumbnail: Option<(String, Option<String>)>,
    fetching: Option<String>,
    last: String,
}
#[derive(Default)]
pub struct Service {
    inner: Mutex<Inner>,
}

/// Returns `(source, video id, clean title)` when the URL is a YouTube or YouTube Music watch page.
fn parse(url: &str, title: &str) -> Option<(&'static str, String, String)> {
    let url = url::Url::parse(url).ok()?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.path() != "/watch"
    {
        return None;
    }
    let (source, suffix) = match url.host_str()? {
        "www.youtube.com" | "youtube.com" | "m.youtube.com" => ("youtube", " - YouTube"),
        "music.youtube.com" => ("music", " - YouTube Music"),
        _ => return None,
    };
    let id = url
        .query_pairs()
        .find(|(key, _)| key == "v")?
        .1
        .into_owned();
    if id.len() != 11
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return None;
    }
    let mut title = title.trim();
    // YouTube prefixes the tab title with the unread notification count: "(3) Title".
    if let Some(rest) = title.strip_prefix('(') {
        if let Some((count, rest)) = rest.split_once(')') {
            if !count.is_empty() && count.bytes().all(|b| b.is_ascii_digit()) {
                title = rest.trim_start();
            }
        }
    }
    let title = title.strip_suffix(suffix).unwrap_or(title).trim();
    Some((source, id, title.chars().take(200).collect()))
}
fn candidate(app: &AppHandle) -> Option<NowPlaying> {
    if super::spotify::private(app) {
        return None;
    }
    app.state::<BrowserState>()
        .snapshot_tabs()
        .into_iter()
        .filter_map(|tab| {
            let (source, video_id, title) = parse(&tab.url, &tab.title)?;
            let progress = app.state::<crate::media::MediaService>().progress(&tab.id);
            let rank = (
                tab.activity.playing_media,
                tab.is_active,
                tab.last_active_epoch,
            );
            Some((
                rank,
                NowPlaying {
                    tab_id: tab.id,
                    source,
                    video_id,
                    title,
                    playing: tab.activity.playing_media,
                    position: progress.map(|(position, _)| position.floor()),
                    duration: progress.map(|(_, duration)| duration.floor()),
                    image: None,
                },
            ))
        })
        .max_by_key(|(rank, _)| *rank)
        .map(|(_, video)| video)
}
/// Current card data, with the thumbnail only when it is already cached.
pub(super) fn snapshot(app: &AppHandle) -> Option<NowPlaying> {
    let mut video = candidate(app)?;
    let service = app.state::<Widgets>();
    let inner = service.youtube.inner.lock().ok()?;
    video.image = inner
        .thumbnail
        .as_ref()
        .filter(|(id, _)| *id == video.video_id)
        .and_then(|(_, image)| image.clone());
    Some(video)
}
fn thumbnail(video_id: &str) -> Option<String> {
    let url = url::Url::parse(&format!("https://i.ytimg.com/vi/{video_id}/mqdefault.jpg")).ok()?;
    let response = super::http::client()
        .ok()?
        .get(url)
        .send()
        .ok()?
        .error_for_status()
        .ok()?;
    let mut bytes = Vec::new();
    response.take(262_145).read_to_end(&mut bytes).ok()?;
    (bytes.len() <= 262_144 && bytes.starts_with(&[0xff, 0xd8, 0xff]))
        .then(|| format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes)))
}
/// Asks the tab to report its playback position; the answer arrives as a media signal.
fn probe(app: &AppHandle, tab_id: &str) {
    if let Some(view) = app.get_webview(&crate::state::tab_model::webview_label_for_tab(tab_id)) {
        crate::error_log::ignored(
            view.eval("window.dispatchEvent(new CustomEvent('spark-media-probe'));"),
        );
    }
}
/// Called while a new tab is visible: fetches a missing thumbnail and pushes changes to the pages.
pub(super) fn refresh(app: &AppHandle) {
    update(app, true);
}
/// Called by the background loop: keeps the vertical-tabs sidebar card current even while no new
/// tab is on screen (a visible new tab is already served by `refresh`).
pub(super) fn tick(app: &AppHandle) {
    let (enabled, sidebar) = settings(app);
    if !enabled || (sidebar && !super::visible(app)) {
        update(app, enabled);
    }
}
fn settings(app: &AppHandle) -> (bool, bool) {
    app.state::<Widgets>()
        .runtime
        .lock()
        .map_or((false, false), |state| {
            (
                state.settings.youtube_enabled,
                state.settings.youtube_sidebar,
            )
        })
}
/// The card the vertical-tabs sidebar shows: only while the widget and its sidebar option are on.
pub(crate) fn sidebar_snapshot(app: &AppHandle) -> Option<NowPlaying> {
    let (enabled, sidebar) = settings(app);
    if enabled && sidebar {
        snapshot(app)
    } else {
        None
    }
}
fn update(app: &AppHandle, ask: bool) {
    let service = app.state::<Widgets>();
    let (enabled, sidebar) = settings(app);
    let Ok(mut inner) = service.youtube.inner.lock() else {
        return;
    };
    if !enabled {
        // One last empty report removes the card from pages and sidebars that still show it.
        let changed = inner.last != "off";
        inner.last = "off".into();
        drop(inner);
        if changed {
            crate::error_log::ignored(crate::security::emit_shell(
                app,
                "youtube-now-playing",
                None::<NowPlaying>,
            ));
            broadcast(app);
        }
        return;
    }
    if let Some(video) = candidate(app) {
        if ask {
            probe(app, &video.tab_id);
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                update(&app, false);
            });
        }
        let known = inner
            .thumbnail
            .as_ref()
            .is_some_and(|(id, _)| *id == video.video_id);
        if !known && inner.fetching.as_deref() != Some(video.video_id.as_str()) {
            inner.fetching = Some(video.video_id.clone());
            let app = app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let image = thumbnail(&video.video_id);
                if let Ok(mut inner) = app.state::<Widgets>().youtube.inner.lock() {
                    inner.thumbnail = Some((video.video_id, image));
                    inner.fetching = None;
                }
                update(&app, false);
            });
        }
    }
    drop(inner);
    let current = snapshot(app);
    let signature = serde_json::to_string(&(&current, sidebar)).unwrap_or_default();
    let Ok(mut inner) = service.youtube.inner.lock() else {
        return;
    };
    if inner.last != signature {
        inner.last = signature;
        drop(inner);
        let _ =
            crate::security::emit_shell(app, "youtube-now-playing", current.filter(|_| sidebar));
        broadcast(app);
    }
}
fn script(operation: &str, source: &str) -> Option<&'static str> {
    Some(match (operation, source) {
        ("youtube_toggle", _) => {
            "(()=>{const v=document.querySelector('video');if(v){v.paused?v.play():v.pause();}})();"
        }
        ("youtube_next", "music") => "document.querySelector('ytmusic-player-bar .next-button')?.click();",
        ("youtube_next", _) => "document.querySelector('.ytp-next-button')?.click();",
        ("youtube_previous", "music") => {
            "(()=>{const b=document.querySelector('ytmusic-player-bar .previous-button');if(b)b.click();else{const v=document.querySelector('video');if(v)v.currentTime=0;}})();"
        }
        ("youtube_previous", _) => {
            "(()=>{const b=document.querySelector('.ytp-prev-button');if(b&&getComputedStyle(b).display!=='none')b.click();else{const v=document.querySelector('video');if(v)v.currentTime=0;}})();"
        }
        _ => return None,
    })
}
pub(super) async fn control(
    app: &AppHandle,
    operation: &str,
    seconds: Option<f64>,
) -> Result<(), String> {
    let enabled = app
        .state::<Widgets>()
        .runtime
        .lock()
        .is_ok_and(|state| state.settings.youtube_enabled);
    if !enabled {
        return Err("cancelled".into());
    }
    let video = candidate(app).ok_or("youtube_not_found")?;
    if operation == "youtube_open" {
        let state = app.state::<BrowserState>();
        crate::state::session_ops::tabs::switch_tab(app.clone(), state, video.tab_id)
            .await
            .map(|_| ())?;
        return Ok(());
    }
    let script = match (operation, seconds) {
        ("youtube_seek", Some(seconds)) if seconds.is_finite() && (0.0..=86_400.0).contains(&seconds) => {
            format!("(()=>{{const v=document.querySelector('video');if(v)v.currentTime={seconds:.2};}})();")
        }
        ("youtube_seek", _) => return Err("invalid_action".into()),
        _ => script(operation, video.source).ok_or("invalid_action")?.to_string(),
    };
    app.get_webview(&crate::state::tab_model::webview_label_for_tab(
        &video.tab_id,
    ))
    .ok_or("youtube_not_found")?
    .eval(&script)
    .map_err(|_| "youtube_not_found")?;
    // The tab needs a moment to report its new playback state.
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(900)).await;
        refresh(&app);
    });
    Ok(())
}

/// Playback controls for the sidebar card: `toggle`, `next`, `previous` or `open`.
pub(crate) async fn sidebar_control(app: &AppHandle, operation: &str) -> Result<(), String> {
    let operation = match operation {
        "toggle" => "youtube_toggle",
        "next" => "youtube_next",
        "previous" => "youtube_previous",
        "open" => "youtube_open",
        _ => return Err("invalid_action".into()),
    };
    if sidebar_snapshot(app).is_none() {
        return Err("youtube_not_found".into());
    }
    control(app, operation, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    const ID: &str = "dQw4w9WgXcQ";
    #[test]
    fn only_https_watch_pages_on_youtube_hosts_are_accepted() {
        let good = parse(
            &format!("https://www.youtube.com/watch?v={ID}&t=5"),
            "(3) Song - YouTube",
        )
        .unwrap();
        assert_eq!(good, ("youtube", ID.into(), "Song".into()));
        let music = parse(
            &format!("https://music.youtube.com/watch?v={ID}"),
            "Track - YouTube Music",
        )
        .unwrap();
        assert_eq!((music.0, music.2.as_str()), ("music", "Track"));
        for url in [
            format!("http://www.youtube.com/watch?v={ID}"),
            format!("https://youtube.com.evil.test/watch?v={ID}"),
            format!("https://www.youtube.com@evil.test/watch?v={ID}"),
            format!("https://www.youtube.com:8443/watch?v={ID}"),
            format!("https://www.youtube.com/shorts?v={ID}"),
            "https://www.youtube.com/watch?v=short".into(),
            "https://www.youtube.com/watch?v=../../etc/pa".into(),
            "https://www.youtube.com/".into(),
        ] {
            assert!(parse(&url, "x").is_none(), "{url}");
        }
    }
    #[test]
    fn titles_are_cleaned_and_bounded() {
        let url = format!("https://www.youtube.com/watch?v={ID}");
        assert_eq!(parse(&url, "(12) A (b) - YouTube").unwrap().2, "A (b)");
        assert_eq!(parse(&url, "(x) Title").unwrap().2, "(x) Title");
        assert_eq!(parse(&url, &"a".repeat(500)).unwrap().2.len(), 200);
    }
    #[test]
    fn controls_exist_for_every_source_and_unknown_operations_are_rejected() {
        for source in ["youtube", "music"] {
            for op in ["youtube_toggle", "youtube_next", "youtube_previous"] {
                assert!(script(op, source).is_some());
            }
        }
        assert!(script("youtube_eval", "youtube").is_none());
        assert!(script("youtube_seek", "youtube").is_none());
    }
}
