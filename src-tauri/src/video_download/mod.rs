//! Finds the videos of a loaded page that Spark can save: direct files and unprotected HLS streams.
//! The page is only asked when the user opens the panel.
pub mod playlist;
mod preview;

use crate::downloads::{hls, transfer::validate_download_url};
use playlist::Playlist;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use tauri::{AppHandle, Manager, Url};

const MAX_PLAYLISTS: usize = 8;
const MAX_SOURCES: usize = 64;
/// The page reports one entry per video element; more than this is not a page anyone saves from.
const MAX_VIDEOS: usize = 256;
/// The popover accepts a limited amount of picture data. Also the most posters requested.
const MAX_THUMBNAILS: usize = 12;
/// How long the frames of the page get, together, to list their sources.
const FRAMES_DEADLINE: Duration = Duration::from_secs(10);
/// How long the streams get, together, to be read. They are read at the same time.
const STREAMS_DEADLINE: Duration = Duration::from_secs(20);
/// How long the posters get, together. They are requested at the same time.
const POSTERS_DEADLINE: Duration = Duration::from_secs(6);

#[derive(Clone, Debug, Deserialize)]
pub struct Found {
    pub url: String,
    pub kind: String,
    #[serde(default)]
    pub height: u32,
    /// The video element or player it belongs to; empty for an address only seen being requested.
    #[serde(default)]
    pub group: String,
}
/// What the page knows about one video.
#[derive(Debug, Default, Deserialize)]
pub struct FoundVideo {
    pub group: String,
    /// Visible area in pixels.
    #[serde(default)]
    pub area: u64,
    #[serde(default)]
    pub duration: u64,
    #[serde(default)]
    pub poster: String,
    #[serde(default)]
    pub frame: String,
}
#[derive(Debug, Deserialize)]
pub struct FrameSources {
    #[serde(default)]
    pub videos: Vec<FoundVideo>,
    #[serde(default)]
    pub page: String,
    #[serde(default)]
    pub protected: bool,
    #[serde(default)]
    pub sources: Vec<Found>,
}
#[derive(Clone, Debug, Serialize)]
pub struct VideoSource {
    pub url: String,
    /// `file` or `hls`.
    pub kind: String,
    pub referer: Option<String>,
    pub host: String,
    pub height: u32,
    pub bandwidth: u64,
    /// Stable code that explains why this source cannot be saved.
    pub problem: Option<String>,
    /// The `PageVideo` this option saves.
    pub video: String,
}
/// One video of the page, with every way of saving it listed in the sources.
#[derive(Debug, Serialize)]
pub struct PageVideo {
    pub id: String,
    pub thumbnail: Option<String>,
    /// Seconds; zero when unknown.
    pub duration: u64,
    pub on_screen: bool,
}
#[derive(Debug, Default, Serialize)]
pub struct VideoScan {
    /// Videos on the screen first.
    pub videos: Vec<PageVideo>,
    pub sources: Vec<VideoSource>,
    /// A video on the page uses content protection.
    pub protected: bool,
}

fn source(url: &Url, kind: &str, referer: &Option<String>, height: u32) -> VideoSource {
    VideoSource {
        url: url.to_string(),
        kind: kind.into(),
        referer: referer.clone(),
        host: url.host_str().unwrap_or_default().into(),
        height,
        bandwidth: 0,
        problem: None,
        video: String::new(),
    }
}

pub fn shield(app: &AppHandle, tab_id: &str, url: &str) -> bool {
    crate::privacy::adblock_engine::download_shield_enabled(
        &app.state::<crate::state::browser_state::BrowserState>(),
        Some(tab_id),
        url,
    )
}

#[cfg(any(windows, target_os = "linux"))]
async fn page_sources(app: &AppHandle, tab_id: &str) -> Result<Vec<FrameSources>, String> {
    use crate::media::{protocol, MediaService};
    let service = app.state::<MediaService>();
    let _operation = service.operations.lock().await;
    let (token, contexts) = protocol::contexts(app, tab_id).await?;
    // The media controls wait for this lock, so slow frames cannot hold it for long.
    let deadline = tokio::time::Instant::now() + FRAMES_DEADLINE;
    let total = contexts.len();
    let mut frames = Vec::new();
    for (index, context) in contexts.into_iter().enumerate() {
        let operation = serde_json::json!({"kind": "sources"});
        let reply = tokio::time::timeout_at(
            deadline,
            protocol::evaluate(app, tab_id, &token, &context, operation, false),
        )
        .await;
        let Ok(reply) = reply else {
            crate::error_log::warn(&format!(
                "Video download: the page's frames took too long to answer; {} of {total} were not read",
                total - index
            ));
            break;
        };
        match reply {
            Ok(value) => match serde_json::from_value(value) {
                Ok(frame) => frames.push(frame),
                Err(error) => crate::error_log::warn(&format!(
                    "Video download: a frame sent an unreadable source list: {error}"
                )),
            },
            // A frame that went away or has no controller has nothing to offer.
            Err(e) if e == "media.unavailable" || e == "media.stale" => {}
            Err(e) => return Err(e),
        }
    }
    Ok(frames)
}
#[cfg(not(any(windows, target_os = "linux")))]
async fn page_sources(_: &AppHandle, _: &str) -> Result<Vec<FrameSources>, String> {
    Err("video.unavailable".into())
}

/// The folder an address is in. The site root says nothing about which video it is.
fn folder(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    let (folder, _) = url.path().rsplit_once('/')?;
    (!folder.is_empty()).then(|| format!("{}{folder}", url.host_str().unwrap_or_default()))
}

/// Gives every source its video. Copies of one video (same folder and length, such as the blurred
/// background some players draw) become one, and an address only seen being requested joins the
/// single video that already uses its folder.
fn assign_videos(sources: &mut [VideoSource], videos: &HashMap<String, FoundVideo>) {
    let duration = |group: &str| videos.get(group).map_or(0, |video| video.duration);
    let mut folders: HashMap<String, Vec<String>> = HashMap::new();
    let mut renamed: HashMap<String, String> = HashMap::new();
    for entry in sources.iter().filter(|entry| !entry.video.is_empty()) {
        let Some(folder) = folder(&entry.url) else {
            continue;
        };
        let groups = folders.entry(folder).or_default();
        let group = renamed.get(&entry.video).unwrap_or(&entry.video).clone();
        if groups.contains(&group) {
            continue;
        }
        let copy = groups
            .iter()
            .find(|other| duration(other) > 0 && duration(other) == duration(&group));
        match copy {
            Some(first) => {
                renamed.insert(entry.video.clone(), first.clone());
            }
            None => groups.push(group),
        }
    }
    for entry in sources.iter_mut() {
        if let Some(group) = renamed.get(&entry.video) {
            entry.video.clone_from(group);
        }
    }
    for entry in sources.iter_mut().filter(|entry| entry.video.is_empty()) {
        let folder = folder(&entry.url);
        entry.video = match folder.as_ref().and_then(|folder| folders.get(folder)) {
            Some(groups) if groups.len() == 1 => groups[0].clone(),
            _ => folder.unwrap_or_else(|| entry.url.clone()),
        };
    }
}

/// A stream read, with the problem of its first complete variant when it is a master playlist.
type StreamRead = Result<(Playlist, Option<String>), String>;

/// Reads a stream. Encryption and live state are only declared inside a variant, so for a master
/// playlist one variant with its own sound is read too.
async fn read_stream(url: Url, shielded: bool, referer: Option<String>) -> StreamRead {
    let playlist = hls::fetch_playlist(&url, shielded, referer.as_deref()).await?;
    let problem = match &playlist {
        Playlist::Master(variants) => match variants.iter().find(|v| !v.separate_audio) {
            Some(variant) => hls::fetch_media(&variant.url, shielded, referer.as_deref())
                .await
                .err(),
            None => None,
        },
        Playlist::Media(_) => None,
    };
    Ok((playlist, problem))
}

/// Reads the first `MAX_PLAYLISTS` streams of the page at the same time, keyed by address. A
/// stream not read before `STREAMS_DEADLINE` is reported as a server that did not answer.
async fn read_streams(
    frames: &[FrameSources],
    shield: &impl Fn(&str) -> bool,
    limit: Duration,
) -> HashMap<String, StreamRead> {
    let mut reads = tokio::task::JoinSet::new();
    let mut pending = HashSet::new();
    for frame in frames {
        let referer = validate_download_url(&frame.page)
            .ok()
            .map(|url| url.to_string());
        for found in frame.sources.iter().filter(|found| found.kind == "hls") {
            if pending.len() >= MAX_PLAYLISTS {
                break;
            }
            let Ok(mut url) = validate_download_url(&found.url) else {
                continue;
            };
            url.set_fragment(None);
            let key = url.to_string();
            if !pending.insert(key.clone()) {
                continue;
            }
            let shielded = shield(url.as_str());
            let referer = referer.clone();
            reads.spawn(async move { (key, read_stream(url, shielded, referer).await) });
        }
    }
    let deadline = tokio::time::Instant::now() + limit;
    let mut read = HashMap::new();
    loop {
        match tokio::time::timeout_at(deadline, reads.join_next()).await {
            Ok(Some(Ok((key, result)))) => {
                read.insert(key, result);
            }
            Ok(Some(Err(error))) => crate::error_log::warn(&format!(
                "Video download: reading a stream stopped unexpectedly: {error}"
            )),
            Ok(None) => break,
            Err(_) => {
                crate::error_log::warn(&format!(
                    "Video download: {} stream(s) did not answer in {:.1} seconds",
                    reads.len(),
                    limit.as_secs_f32()
                ));
                reads.abort_all();
                break;
            }
        }
    }
    for key in pending {
        read.entry(key)
            .or_insert_with(|| Err("downloads.network_error".into()));
    }
    read
}

/// Turns what the frames reported into the list the panel shows. Streams are read to list their
/// qualities; the variants of a listed master playlist are not repeated on their own.
async fn resolve(
    frames: Vec<FrameSources>,
    shield: impl Fn(&str) -> bool,
) -> (VideoScan, HashMap<String, FoundVideo>) {
    let mut streams = read_streams(&frames, &shield, STREAMS_DEADLINE).await;
    let mut scan = VideoScan::default();
    let mut videos = HashMap::new();
    let mut seen = HashSet::new();
    let mut variant_urls = HashSet::new();
    // Entries listed through their master playlist, which knows whether the sound is separate.
    let mut from_master = Vec::new();
    for frame in frames {
        scan.protected |= frame.protected;
        for video in frame.videos {
            if videos.len() < MAX_VIDEOS {
                videos.insert(video.group.clone(), video);
            }
        }
        let referer = validate_download_url(&frame.page)
            .ok()
            .map(|url| url.to_string());
        for found in frame.sources {
            let Ok(mut url) = validate_download_url(&found.url) else {
                continue;
            };
            url.set_fragment(None);
            if scan.sources.len() >= MAX_SOURCES || !seen.insert(url.to_string()) {
                continue;
            }
            if found.kind != "hls" {
                let mut entry = source(&url, "file", &referer, found.height);
                entry.video = found.group;
                scan.sources.push(entry);
                from_master.push(false);
                continue;
            }
            // Only the first `MAX_PLAYLISTS` streams were read.
            let Some(fetched) = streams.remove(url.as_str()) else {
                continue;
            };
            match fetched {
                Ok((Playlist::Master(variants), problem)) => {
                    for variant in variants {
                        variant_urls.insert(variant.url.to_string());
                        let mut entry = source(&variant.url, "hls", &referer, variant.height);
                        entry.video.clone_from(&found.group);
                        entry.bandwidth = variant.bandwidth;
                        entry.problem = if variant.separate_audio {
                            Some("video.separate_audio".into())
                        } else {
                            problem.clone()
                        };
                        scan.sources.push(entry);
                        from_master.push(true);
                    }
                }
                other => {
                    let mut entry = source(&url, "hls", &referer, found.height);
                    entry.video = found.group;
                    entry.problem = other.err();
                    if let Some(problem) = &entry.problem {
                        crate::error_log::report(
                            crate::error_log::Level::Info,
                            "video-download",
                            "A stream cannot be saved",
                            &format!("{}: {problem}", entry.host),
                        );
                    }
                    scan.sources.push(entry);
                    from_master.push(false);
                }
            }
        }
    }
    // A player usually loads a variant of its master playlist too. Only the entry listed through
    // the master is kept: on its own a variant would look complete even when it has no sound.
    let mut listed = HashSet::new();
    let mut origin = from_master.into_iter();
    scan.sources.retain(|entry| {
        let from_master = origin.next().unwrap_or(false);
        !variant_urls.contains(&entry.url) || (from_master && listed.insert(entry.url.clone()))
    });
    assign_videos(&mut scan.sources, &videos);
    // In the order the page gave them (on screen first); what can be saved and the best quality first.
    let mut order: Vec<String> = Vec::new();
    for entry in &scan.sources {
        if !order.contains(&entry.video) {
            order.push(entry.video.clone());
        }
    }
    let area = |id: &str| videos.get(id).map_or(0, |video| video.area);
    order.sort_by_key(|id| std::cmp::Reverse(area(id)));
    scan.sources.sort_by_key(|entry| {
        (
            order.iter().position(|id| *id == entry.video),
            entry.problem.is_some(),
            std::cmp::Reverse((entry.height, entry.bandwidth)),
        )
    });
    scan.videos = order
        .into_iter()
        .map(|id| PageVideo {
            duration: videos.get(&id).map_or(0, |video| video.duration),
            on_screen: area(&id) > 0,
            thumbnail: None,
            id,
        })
        .collect();
    (scan, videos)
}

/// A frame the page drew, or the poster it names. A missing picture never fails the search.
async fn add_thumbnails(
    app: &AppHandle,
    tab_id: &str,
    scan: &mut VideoScan,
    videos: &HashMap<String, FoundVideo>,
) {
    // Every attempt counts, not only the pictures made, so a page full of broken posters cannot
    // keep the panel waiting.
    let mut attempts = 0;
    let mut posters = tokio::task::JoinSet::new();
    for (index, video) in scan.videos.iter_mut().enumerate() {
        if attempts >= MAX_THUMBNAILS {
            break;
        }
        let Some(found) = videos.get(&video.id) else {
            continue;
        };
        if !found.frame.is_empty() {
            attempts += 1;
            video.thumbnail = preview::from_frame(&found.frame);
            if video.thumbnail.is_some() {
                continue;
            }
        }
        let Ok(url) = validate_download_url(&found.poster) else {
            continue;
        };
        attempts += 1;
        let shielded = shield(app, tab_id, url.as_str());
        posters.spawn(async move {
            let picture = hls::get(&url, shielded, None, preview::MAX_POSTER_BYTES).await;
            (index, picture)
        });
    }
    let deadline = tokio::time::Instant::now() + POSTERS_DEADLINE;
    loop {
        match tokio::time::timeout_at(deadline, posters.join_next()).await {
            Ok(Some(Ok((index, Ok((_, bytes)))))) => {
                scan.videos[index].thumbnail = preview::from_poster(&bytes);
            }
            Ok(Some(Ok((_, Err(error))))) => crate::error_log::warn(&format!(
                "Video download: a poster picture could not be read: {error}"
            )),
            Ok(Some(Err(error))) => crate::error_log::warn(&format!(
                "Video download: reading a poster picture stopped unexpectedly: {error}"
            )),
            Ok(None) => break,
            Err(_) => {
                crate::error_log::warn(&format!(
                    "Video download: {} poster picture(s) did not arrive in time",
                    posters.len()
                ));
                posters.abort_all();
                break;
            }
        }
    }
}

pub async fn scan(app: &AppHandle, tab_id: &str) -> Result<VideoScan, String> {
    let frames = page_sources(app, tab_id).await?;
    let (mut scan, videos) = resolve(frames, |url| shield(app, tab_id, url)).await;
    add_thumbnails(app, tab_id, &mut scan, &videos).await;
    Ok(scan)
}

/// Windows refuses these names with any extension.
fn reserved(stem: &str) -> bool {
    let upper = stem
        .split('.')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.ends_with(|c: char| c.is_ascii_digit()))
}

/// A file name made from the page title.
pub fn file_name(title: &str, extension: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.' | '(' | ')') {
                c
            } else {
                ' '
            }
        })
        .collect();
    let stem: String = cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect();
    let stem = stem.trim_matches(|c| c == '.' || c == ' ');
    if stem.is_empty() {
        format!("video.{extension}")
    } else if reserved(stem) {
        format!("video-{stem}.{extension}")
    } else {
        format!("{stem}.{extension}")
    }
}

pub fn file_extension(url: &Url) -> &'static str {
    let path = url.path().to_ascii_lowercase();
    ["mp4", "webm", "ogv", "mov", "mkv", "m4v"]
        .into_iter()
        .find(|extension| path.ends_with(&format!(".{extension}")))
        .unwrap_or("mp4")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_safe_file_names() {
        assert_eq!(
            file_name("Aula 1: Introdução / parte?", "mp4"),
            "Aula 1 Introdução parte.mp4"
        );
        assert_eq!(file_name("..\\..", "ts"), "video.ts");
        assert_eq!(file_name("con", "ts"), "video-con.ts");
        assert_eq!(file_name(&"a".repeat(300), "mp4").len(), 84);
        let url = Url::parse("https://example.test/a/movie.WEBM?x=1").unwrap();
        assert_eq!(file_extension(&url), "webm");
        assert_eq!(
            file_extension(&Url::parse("https://example.test/get?id=1").unwrap()),
            "mp4"
        );
    }

    #[tokio::test]
    async fn lists_files_once_and_ignores_unsafe_addresses() {
        let found = |url: &str| Found {
            url: url.into(),
            kind: "file".into(),
            height: 720,
            group: String::new(),
        };
        let (scan, _) = resolve(
            vec![FrameSources {
                videos: Vec::new(),
                page: "about:blank".into(),
                protected: true,
                sources: vec![
                    found("https://example.test/a.mp4#t=1"),
                    found("https://example.test/a.mp4"),
                    found("file:///c:/a.mp4"),
                    found("https://user:secret@example.test/b.mp4"),
                ],
            }],
            |_| true,
        )
        .await;
        assert!(scan.protected);
        assert_eq!(scan.sources.len(), 1);
        assert_eq!(scan.sources[0].url, "https://example.test/a.mp4");
        assert_eq!(scan.sources[0].referer, None);
    }

    #[tokio::test]
    async fn streams_are_read_together_and_a_silent_server_ends_at_the_deadline() {
        // Accepts connections and never answers.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let held = std::thread::spawn(move || {
            (0..2)
                .map(|_| listener.accept().unwrap().0)
                .collect::<Vec<_>>()
        });
        let stream = |name: &str| Found {
            url: format!("http://{address}/{name}.m3u8"),
            kind: "hls".into(),
            height: 0,
            group: String::new(),
        };
        let frames = vec![FrameSources {
            videos: Vec::new(),
            page: String::new(),
            protected: false,
            sources: vec![stream("a"), stream("b"), stream("a")],
        }];
        let started = std::time::Instant::now();
        let read = read_streams(&frames, &|_| false, Duration::from_millis(400)).await;
        // Read one after the other, two silent servers would take twice the deadline.
        assert!(started.elapsed() < Duration::from_millis(750));
        assert_eq!(read.len(), 2);
        assert!(read
            .values()
            .all(|result| result.as_ref().err().map(String::as_str)
                == Some("downloads.network_error")));
        drop(held.join().unwrap());
    }

    #[test]
    fn groups_the_options_of_one_video_without_mixing_different_ones() {
        let entry = |url: &str, video: &str| {
            let mut entry = source(&Url::parse(url).unwrap(), "file", &None, 0);
            entry.video = video.into();
            entry
        };
        let found = |group: &str, duration| {
            (
                group.to_string(),
                FoundVideo {
                    group: group.into(),
                    duration,
                    ..Default::default()
                },
            )
        };
        let videos = HashMap::from([found("a", 30), found("a-blur", 30), found("b", 12)]);
        let mut sources = vec![
            entry("https://v.example.test/one/small.mp4", "a"),
            entry("https://v.example.test/one/tiny.mp4", "a-blur"),
            entry("https://v.example.test/one/list.m3u8", ""),
            entry("https://v.example.test/shared/b.mp4", "b"),
            entry("https://v.example.test/shared/a.mp4", "a"),
            entry("https://v.example.test/shared/which.m3u8", ""),
            entry("https://v.example.test/two/x.m3u8", ""),
            entry("https://v.example.test/two/y.m3u8", ""),
        ];
        assign_videos(&mut sources, &videos);
        let ids: Vec<_> = sources.iter().map(|s| s.video.as_str()).collect();
        assert_eq!(&ids[..5], ["a", "a", "a", "b", "a"]);
        // Two videos share this folder, so the request cannot be given to either.
        assert_eq!(ids[5], "v.example.test/shared");
        assert_eq!(ids[6], ids[7]);
    }
}
