use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock,
    },
    time::{Duration, Instant},
};
use tauri::Url;

/// Holds a download back so it does not go faster than the limit chosen in the resources panel (0 is none).
/// The wait is cut in small pieces so a cancel still takes effect at once.
fn throttle(started: Instant, received: u64, cancel: &AtomicBool) {
    let limit = crate::resources::DOWNLOAD_LIMIT.load(Ordering::Relaxed);
    if limit == 0 {
        return;
    }
    let due = Duration::from_secs_f64(received as f64 / limit as f64);
    while started.elapsed() < due && !cancel.load(Ordering::Relaxed) {
        std::thread::sleep((due - started.elapsed()).min(Duration::from_millis(100)));
    }
}

pub struct SavedDownload {
    pub file_name: String,
    pub path: PathBuf,
    pub bytes: u64,
}

pub fn validate_download_url(raw: &str) -> Result<Url, String> {
    let url = Url::parse(raw.trim()).map_err(|_| "Enter a valid HTTP or HTTPS download URL")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Downloads require an HTTP or HTTPS URL without embedded credentials".into());
    }
    Ok(url)
}

pub fn file_name(url: &Url) -> String {
    let raw = url
        .path_segments()
        .and_then(|mut parts| parts.next_back())
        .unwrap_or("");
    let sanitized: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .take(160)
        .collect();
    if sanitized.is_empty() || sanitized.trim_matches('.').is_empty() {
        "download.html".into()
    } else {
        // A prefix also avoids Windows device names such as CON and NUL.
        format!("download-{sanitized}")
    }
}

fn shared_client(shield_enabled: bool) -> Result<&'static reqwest::blocking::Client, String> {
    static ENABLED: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();
    static DISABLED: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();
    (if shield_enabled { &ENABLED } else { &DISABLED })
        .get_or_init(|| build_client(shield_enabled))
        .as_ref()
        .map_err(Clone::clone)
}

fn build_client(shield_enabled: bool) -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= 10 {
                attempt.error("Too many download redirects")
            } else if validate_download_url(attempt.url().as_str()).is_err()
                || crate::privacy::adblock_engine::should_block_url(
                    attempt.url().as_str(),
                    shield_enabled,
                )
            {
                attempt.error("Download redirect was blocked")
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| e.to_string())
}

pub fn download_to_directory(
    url: &Url,
    destination: &Path,
    shield_enabled: bool,
) -> Result<SavedDownload, String> {
    download_with_control(
        url,
        destination,
        shield_enabled,
        &AtomicBool::new(false),
        |_, _| {},
    )
}

pub fn download_with_control(
    url: &Url,
    destination: &Path,
    shield_enabled: bool,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64, Option<u64>),
) -> Result<SavedDownload, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("Download cancelled".into());
    }
    validate_download_url(url.as_str())?;
    if crate::privacy::adblock_engine::should_block_url(url.as_str(), shield_enabled) {
        return Err("Download blocked by the privacy shield".into());
    }
    std::fs::create_dir_all(destination)
        .map_err(|e| format!("Could not open Downloads folder: {e}"))?;
    let client = shared_client(shield_enabled)?;
    let mut response = client
        .get(url.clone())
        .send()
        .map_err(|_| "Download request failed; check the URL and connection")?
        .error_for_status()
        .map_err(|e| {
            format!(
                "Download server returned {}",
                e.status().map(|s| s.as_u16()).unwrap_or(0)
            )
        })?;
    let mut temporary = tempfile::NamedTempFile::new_in(destination).map_err(|e| e.to_string())?;
    let total = response.content_length();
    let mut received = 0;
    let started = Instant::now();
    let mut buffer = [0; 64 * 1024];
    progress(0, total);
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Download cancelled".into());
        }
        if started.elapsed() > Duration::from_secs(3600) {
            return Err("Download time limit exceeded".into());
        }
        let count = response
            .read(&mut buffer)
            .map_err(|_| "Download interrupted before completion")?;
        if count == 0 {
            break;
        }
        temporary
            .as_file_mut()
            .write_all(&buffer[..count])
            .map_err(|e| e.to_string())?;
        received += count as u64;
        progress(received, total);
        throttle(started, received, cancel);
    }
    if cancel.load(Ordering::Relaxed) {
        return Err("Download cancelled".into());
    }
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    let base_name = file_name(response.url());
    for number in 0..10_000 {
        let name = if number == 0 {
            base_name.clone()
        } else {
            format!("{number}-{base_name}")
        };
        let path = destination.join(&name);
        match temporary.persist_noclobber(&path) {
            Ok(_) => {
                return Ok(SavedDownload {
                    file_name: name,
                    path,
                    bytes: received,
                })
            }
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                temporary = error.file
            }
            Err(error) => return Err(format!("Could not save downloaded file: {}", error.error)),
        }
    }
    Err("Too many downloads with the same name".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    #[test]
    fn cancellation_removes_partial_files_and_client_pools_are_reused() {
        let directory = tempfile::tempdir().unwrap();
        let cancel = AtomicBool::new(false);
        let url = server("200 OK", &"a".repeat(200_000));
        let result = download_with_control(&url, directory.path(), true, &cancel, |received, _| {
            if received > 0 {
                cancel.store(true, Ordering::Relaxed);
            }
        });
        assert_eq!(result.err().as_deref(), Some("Download cancelled"));
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        assert!(std::ptr::eq(
            shared_client(true).unwrap(),
            shared_client(true).unwrap()
        ));
        assert!(!std::ptr::eq(
            shared_client(true).unwrap(),
            shared_client(false).unwrap()
        ));
    }

    fn server(status: &str, body: &str) -> Url {
        raw_server(format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ))
    }

    fn raw_server(response: String) -> Url {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = Url::parse(&format!(
            "http://{}/sample.txt",
            listener.local_addr().unwrap()
        ))
        .unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                stream.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            stream.write_all(response.as_bytes()).unwrap();
        });
        url
    }

    #[test]
    fn saves_bytes_without_overwriting_existing_files() {
        let directory = tempfile::tempdir().unwrap();
        let first =
            download_to_directory(&server("200 OK", "first"), directory.path(), true).unwrap();
        let second =
            download_to_directory(&server("200 OK", "second"), directory.path(), true).unwrap();
        assert_ne!(first.path, second.path);
        assert_eq!(std::fs::read(first.path).unwrap(), b"first");
        assert_eq!(std::fs::read(second.path).unwrap(), b"second");
    }

    #[test]
    fn failed_http_response_leaves_no_completed_file() {
        let directory = tempfile::tempdir().unwrap();
        assert!(
            download_to_directory(&server("404 Not Found", "missing"), directory.path(), true)
                .is_err()
        );
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        assert!(validate_download_url("file:///local.txt").is_err());
        assert!(validate_download_url("https://name:password@example.test/").is_err());
    }

    #[test]
    fn validates_redirects_before_contacting_the_destination() {
        let directory = tempfile::tempdir().unwrap();
        let denied = raw_server("HTTP/1.1 302 Found\r\nLocation: https://google-analytics.com/collect\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into());
        assert!(download_to_directory(&denied, directory.path(), true).is_err());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        let target = server("200 OK", "redirected");
        let allowed = raw_server(format!("HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"));
        let saved = download_to_directory(&allowed, directory.path(), true).unwrap();
        assert_eq!(std::fs::read(saved.path).unwrap(), b"redirected");
    }

    #[test]
    fn interrupted_response_removes_the_temporary_file() {
        let directory = tempfile::tempdir().unwrap();
        let url = raw_server(
            "HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nshort".into(),
        );
        assert!(download_to_directory(&url, directory.path(), true).is_err());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
    }
}
