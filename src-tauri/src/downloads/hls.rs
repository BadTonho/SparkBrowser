//! Saves an unprotected HLS stream: its segments are joined, in order, into one partial file.
use super::{
    files,
    manager::{Control, DownloadManager},
    model::DownloadRecord,
    resumable,
};
use crate::video_download::playlist::{self, MediaPlaylist, Playlist, MAX_PLAYLIST_BYTES};
use sha2::{Digest, Sha256};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, Url};
use tokio::io::AsyncWriteExt;

/// A segment is held in memory so a dropped connection can be retried without damaging the file.
const MAX_SEGMENT_BYTES: usize = 256 * 1024 * 1024;
const ATTEMPTS: u32 = 3;

pub(crate) async fn get(
    url: &Url,
    shield: bool,
    referer: Option<&str>,
    limit: usize,
) -> Result<(Url, Vec<u8>), String> {
    super::transfer::validate_download_url(url.as_str())?;
    if crate::privacy::adblock_engine::should_block_url(url.as_str(), shield) {
        return Err("downloads.blocked".into());
    }
    let request = resumable::client(shield)?
        .get(url.clone())
        .header(reqwest::header::ACCEPT_ENCODING, "identity");
    let mut response = resumable::with_referer(request, url, referer)
        .send()
        .await
        .map_err(|_| "downloads.network_error")?;
    if response.status() != reqwest::StatusCode::OK {
        return Err("downloads.http_error".into());
    }
    let final_url =
        Url::parse(response.url().as_str()).map_err(|_| "downloads.network_error".to_string())?;
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "downloads.network_error")?
    {
        if body.len() + chunk.len() > limit {
            return Err("video.too_large".into());
        }
        body.extend_from_slice(&chunk);
    }
    Ok((final_url, body))
}

pub async fn fetch_playlist(
    url: &Url,
    shield: bool,
    referer: Option<&str>,
) -> Result<Playlist, String> {
    let (final_url, body) = tokio::time::timeout(
        Duration::from_secs(15),
        get(url, shield, referer, MAX_PLAYLIST_BYTES),
    )
    .await
    .map_err(|_| "downloads.network_error")??;
    playlist::parse(&String::from_utf8_lossy(&body), &final_url).map_err(str::to_string)
}

pub async fn fetch_media(
    url: &Url,
    shield: bool,
    referer: Option<&str>,
) -> Result<MediaPlaylist, String> {
    match fetch_playlist(url, shield, referer).await? {
        Playlist::Media(media) => Ok(media),
        Playlist::Master(_) => Err("video.invalid_playlist".into()),
    }
}

async fn segment(
    url: &Url,
    shield: bool,
    referer: Option<&str>,
    control: &Control,
) -> Result<Vec<u8>, String> {
    let mut attempt = 1;
    loop {
        let result = tokio::select! {biased;
            _ = control.changed.notified() => return Err("downloads.stopped".into()),
            result = get(url, shield, referer, MAX_SEGMENT_BYTES) => result,
        };
        match result {
            Ok((_, body)) => return Ok(body),
            Err(error) if error == "downloads.network_error" && attempt < ATTEMPTS => {
                tokio::select! {biased;
                    _ = control.changed.notified() => return Err("downloads.stopped".into()),
                    _ = tokio::time::sleep(Duration::from_secs(u64::from(attempt))) => {}
                }
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

/// Writes every segment to `file` and returns the size and hash of the result.
async fn join(
    media: &MediaPlaylist,
    file: &mut tokio::fs::File,
    shield: bool,
    referer: Option<&str>,
    control: &Control,
    mut progress: impl FnMut(u64),
) -> Result<(u64, Sha256), String> {
    let mut hash = Sha256::new();
    let mut count = 0;
    for url in media.init.iter().chain(&media.segments) {
        if control.requested.load(Ordering::Acquire) != 0 {
            return Err("downloads.stopped".into());
        }
        let body = segment(url, shield, referer, control).await?;
        file.write_all(&body).await.map_err(|e| e.to_string())?;
        hash.update(&body);
        count += body.len() as u64;
        progress(count);
    }
    file.sync_all().await.map_err(|e| e.to_string())?;
    Ok((count, hash))
}

/// A stream cannot be resumed: every run starts the file over.
pub async fn run(app: &AppHandle, id: &str, control: &Control) -> Result<(), String> {
    let manager = app.state::<DownloadManager>();
    let record: DownloadRecord = manager.record(id)?;
    let path = files::owned_partial(&record)?;
    let shield = resumable::shield_for(app, &record)?;
    let referer = record.referer.as_deref();
    let url = super::transfer::validate_download_url(&record.task.url)?;
    let media = tokio::select! {biased;
        _ = control.changed.notified() => return Err("downloads.stopped".into()),
        media = fetch_media(&url, shield, referer) => media?,
    };
    manager.edit(app, id, true, |r| {
        r.task.received = 0;
        r.task.total = None;
        r.task.status = "Downloading".into();
    })?;
    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(|e| e.to_string())?;
    let mut shown = Instant::now();
    let (count, hash) = join(&media, &mut file, shield, referer, control, |count| {
        if shown.elapsed() > Duration::from_millis(100) {
            manager.update(app, id, |t| t.received = count);
            shown = Instant::now();
        }
    })
    .await?;
    drop(file);
    if control.requested.load(Ordering::Acquire) != 0 {
        return Err("downloads.stopped".into());
    }
    resumable::finalize(app, id, &record, count, hash).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    /// Answers each connection with the next response and reports the request headers.
    fn server(
        responses: Vec<(&'static str, &'static str)>,
    ) -> (Url, std::thread::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = Url::parse(&format!(
            "http://{}/index.m3u8",
            listener.local_addr().unwrap()
        ))
        .unwrap();
        let handle = std::thread::spawn(move || {
            responses
                .into_iter()
                .map(|(status, body)| {
                    let (mut stream, _) = listener.accept().unwrap();
                    let mut request = Vec::new();
                    while !request.ends_with(b"\r\n\r\n") {
                        let mut byte = [0];
                        stream.read_exact(&mut byte).unwrap();
                        request.push(byte[0]);
                    }
                    write!(stream, "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
                    String::from_utf8(request).unwrap().to_ascii_lowercase()
                })
                .collect()
        });
        (url, handle)
    }

    #[tokio::test]
    async fn joins_segments_in_order_sending_the_page_as_referer() {
        let (url, requests) = server(vec![
            (
                "200 OK",
                "#EXTM3U\n#EXTINF:1,\na.ts\n#EXTINF:1,\nb.ts\n#EXT-X-ENDLIST\n",
            ),
            ("200 OK", "first-"),
            ("200 OK", "second"),
        ]);
        let referer = Some("http://page.example.test/watch?private=1");
        let media = fetch_media(&url, true, referer).await.unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("joined.ts");
        let mut file = tokio::fs::File::create(&path).await.unwrap();
        let mut seen = Vec::new();
        let (count, _) = join(&media, &mut file, true, referer, &Control::default(), |n| {
            seen.push(n)
        })
        .await
        .unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"first-second");
        assert_eq!((count, seen), (12, vec![6, 12]));
        for request in requests.join().unwrap() {
            // Another origin only learns the page's origin.
            assert!(request.contains("referer: http://page.example.test/\r\n"));
            assert!(request.contains("user-agent: mozilla/5.0"));
        }
    }

    #[tokio::test]
    async fn a_missing_segment_or_a_cancel_stops_the_stream() {
        let (url, requests) = server(vec![("404 Not Found", "")]);
        let media = MediaPlaylist {
            init: None,
            segments: vec![url],
        };
        let directory = tempfile::tempdir().unwrap();
        let mut file = tokio::fs::File::create(directory.path().join("a.ts"))
            .await
            .unwrap();
        let control = Control::default();
        let error = join(&media, &mut file, true, None, &control, |_| {}).await;
        assert_eq!(error.err().as_deref(), Some("downloads.http_error"));
        requests.join().unwrap();
        control.requested.store(2, Ordering::Release);
        let error = join(&media, &mut file, true, None, &control, |_| {}).await;
        assert_eq!(error.err().as_deref(), Some("downloads.stopped"));
    }
}
