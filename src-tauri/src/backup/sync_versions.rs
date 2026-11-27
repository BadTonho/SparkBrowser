//! Earlier versions of the file in the cloud, so a bad send never leaves only one copy.
//!
//! - Folder and WebDAV: before a changed file replaces the current one, the current one is copied
//!   to a `versions` folder next to it as `browser.spark-backup.<unix seconds>.json`; the newest
//!   [`KEEP`] copies stay.
//! - Google Drive keeps earlier versions (revisions) of the same file on its own; the bridge lists
//!   and downloads them (see `gdrive_webview`).
use super::{files, model::BackupDocument};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// How many earlier copies a folder or WebDAV target keeps.
pub const KEEP: usize = 5;
pub const DIR: &str = "versions";
const PREFIX: &str = "browser.spark-backup.";
const SUFFIX: &str = ".json";

/// One earlier version of the file in the cloud.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SyncVersion {
    /// The copy's file name (folder, WebDAV) or the Drive revision id.
    pub id: String,
    /// Unix seconds: when that version was written.
    pub created_at: i64,
    pub size: Option<u64>,
}

pub fn version_name(created_at: i64) -> String {
    format!("{PREFIX}{created_at}{SUFFIX}")
}

/// The time in a copy's name; `None` for anything that is not a version copy. Ids from the interface
/// pass through here before they reach a path or a URL, so nothing else can be read.
pub fn parse_version(name: &str) -> Option<i64> {
    let digits = name.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    if digits.is_empty() || digits.len() > 12 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// A Drive revision id from the interface: letters, digits, `-` and `_` only.
pub fn valid_revision_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 200
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Newest first, the copies beyond [`KEEP`] last.
fn newest_first(mut versions: Vec<SyncVersion>) -> Vec<SyncVersion> {
    versions.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| b.id.cmp(&a.id))
    });
    versions
}

// --- Folder -------------------------------------------------------------------------------------

fn folder_versions_dir(file_path: &Path) -> Result<PathBuf, String> {
    Ok(file_path
        .parent()
        .ok_or("backup.sync_folder_missing")?
        .join(DIR))
}

/// Copies the current file into `versions/` (named after when it was written) and keeps the newest
/// [`KEEP`]. Nothing to do when there is no current file yet.
pub fn archive_folder(file_path: &Path) -> Result<(), String> {
    if !file_path.is_file() {
        return Ok(());
    }
    let written = std::fs::metadata(file_path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or_else(
            || crate::state::tab_model::TabInfo::current_epoch_secs() as i64,
            |d| d.as_secs() as i64,
        );
    let dir = folder_versions_dir(file_path)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("backup.sync_versions_failed: {e}"))?;
    std::fs::copy(file_path, dir.join(version_name(written)))
        .map_err(|e| format!("backup.sync_versions_failed: {e}"))?;
    for old in list_folder(file_path)?.into_iter().skip(KEEP) {
        if let Err(error) = std::fs::remove_file(dir.join(&old.id)) {
            crate::error_log::report(
                crate::error_log::Level::Warning,
                "backup",
                "an old sync version could not be removed from the folder",
                &format!("{}: {error}", old.id),
            );
        }
    }
    Ok(())
}

pub fn list_folder(file_path: &Path) -> Result<Vec<SyncVersion>, String> {
    let dir = folder_versions_dir(file_path)?;
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let versions = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            let created_at = parse_version(&name)?;
            let size = entry
                .metadata()
                .ok()
                .filter(|m| m.is_file())
                .map(|m| m.len())?;
            Some(SyncVersion {
                id: name,
                created_at,
                size: Some(size),
            })
        })
        .collect();
    Ok(newest_first(versions))
}

pub fn read_folder_version(
    file_path: &Path,
    id: &str,
    password: Option<&str>,
) -> Result<BackupDocument, String> {
    parse_version(id).ok_or("backup.sync_version_not_found")?;
    let path = folder_versions_dir(file_path)?.join(id);
    if !path.is_file() {
        return Err("backup.sync_version_not_found".into());
    }
    files::read_with(&path, password)
}

// --- WebDAV -------------------------------------------------------------------------------------

/// The `versions/` collection next to the sync file, and the URL of one copy in it.
fn webdav_versions_url(file_url: &url::Url) -> Result<url::Url, String> {
    let mut dir = file_url.clone();
    let path = file_url.path();
    let parent = &path[..path.rfind('/').map_or(0, |i| i + 1)];
    dir.set_path(&format!("{parent}{DIR}/"));
    Ok(dir)
}

fn webdav_version_url(file_url: &url::Url, id: &str) -> Result<url::Url, String> {
    parse_version(id).ok_or("backup.sync_version_not_found")?;
    webdav_versions_url(file_url)?
        .join(id)
        .map_err(|_| "backup.sync_invalid_url".to_string())
}

type Request = dyn Fn(reqwest::Method, url::Url) -> reqwest::blocking::RequestBuilder;

fn status_error(status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        401 | 403 => "backup.sync_auth_failed".into(),
        code => format!("backup.sync_versions_failed: HTTP {code}"),
    }
}

/// Copies the current WebDAV file into `versions/` (server side, with COPY) and keeps the newest
/// [`KEEP`]. A missing current file is not an error.
pub fn archive_webdav(file_url: &url::Url, request: &Request) -> Result<(), String> {
    let dir = webdav_versions_url(file_url)?;
    let mkcol = reqwest::Method::from_bytes(b"MKCOL").map_err(|e| e.to_string())?;
    let created = request(mkcol, dir.clone())
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    // 405: the collection already exists.
    if !created.status().is_success() && created.status().as_u16() != 405 {
        return Err(status_error(created.status()));
    }
    let now = crate::state::tab_model::TabInfo::current_epoch_secs() as i64;
    let target = dir
        .join(&version_name(now))
        .map_err(|_| "backup.sync_invalid_url".to_string())?;
    let copy = reqwest::Method::from_bytes(b"COPY").map_err(|e| e.to_string())?;
    let copied = request(copy, file_url.clone())
        .header("Destination", target.as_str())
        .header("Overwrite", "T")
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    if copied.status().as_u16() == 404 {
        return Ok(());
    }
    if !copied.status().is_success() {
        return Err(status_error(copied.status()));
    }
    for old in list_webdav(file_url, request)?.into_iter().skip(KEEP) {
        let url = webdav_version_url(file_url, &old.id)?;
        match request(reqwest::Method::DELETE, url).send() {
            Ok(response) if response.status().is_success() => {}
            Ok(response) => crate::error_log::report(
                crate::error_log::Level::Warning,
                "backup",
                "an old sync version could not be removed from WebDAV",
                &format!("{}: HTTP {}", old.id, response.status().as_u16()),
            ),
            Err(error) => crate::error_log::report(
                crate::error_log::Level::Warning,
                "backup",
                "an old sync version could not be removed from WebDAV",
                &format!("{}: {error}", old.id),
            ),
        }
    }
    Ok(())
}

pub fn list_webdav(file_url: &url::Url, request: &Request) -> Result<Vec<SyncVersion>, String> {
    let dir = webdav_versions_url(file_url)?;
    let propfind = reqwest::Method::from_bytes(b"PROPFIND").map_err(|e| e.to_string())?;
    let response = request(propfind, dir)
        .header("Depth", "1")
        .header("Content-Type", "application/xml")
        .body(r#"<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:getcontentlength/></d:prop></d:propfind>"#)
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    if response.status().as_u16() == 404 {
        return Ok(Vec::new());
    }
    if !response.status().is_success() {
        return Err(status_error(response.status()));
    }
    let body = response
        .text()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    Ok(newest_first(parse_propfind(&body)))
}

/// The version copies named in a PROPFIND answer (any namespace prefix).
pub fn parse_propfind(xml: &str) -> Vec<SyncVersion> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut versions = Vec::new();
    let (mut in_href, mut in_length) = (false, false);
    let mut current: Option<(String, i64)> = None;
    let mut size: Option<u64> = None;
    loop {
        match reader.read_event() {
            Ok(Event::Start(tag)) => match tag.local_name().as_ref() {
                "response" => {
                    current = None;
                    size = None;
                }
                "href" => in_href = true,
                "getcontentlength" => in_length = true,
                _ => {}
            },
            Ok(Event::Text(text)) => {
                let value = quick_xml::escape::unescape(text.as_ref())
                    .map(|v| v.into_owned())
                    .unwrap_or_default();
                if in_href {
                    let last = value
                        .trim_end_matches('/')
                        .rsplit('/')
                        .next()
                        .unwrap_or_default()
                        .to_string();
                    current = parse_version(&last).map(|time| (last, time));
                } else if in_length {
                    size = value.trim().parse().ok();
                }
            }
            Ok(Event::End(tag)) => match tag.local_name().as_ref() {
                "href" => in_href = false,
                "getcontentlength" => in_length = false,
                "response" => {
                    if let Some((id, created_at)) = current.take() {
                        versions.push(SyncVersion {
                            id,
                            created_at,
                            size,
                        });
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    versions
}

pub fn read_webdav_version(
    file_url: &url::Url,
    id: &str,
    request: &Request,
    password: Option<&str>,
) -> Result<BackupDocument, String> {
    use std::io::Read;
    let url = webdav_version_url(file_url, id)?;
    let response = request(reqwest::Method::GET, url)
        .send()
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    if response.status().as_u16() == 404 {
        return Err("backup.sync_version_not_found".into());
    }
    if !response.status().is_success() {
        return Err(status_error(response.status()));
    }
    let mut bytes = Vec::new();
    response
        .take(files::MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("backup.sync_network_failed: {e}"))?;
    files::decode_with(&bytes, password)
}

#[cfg(test)]
mod tests {
    use super::super::sync::SYNC_FILE_NAME;
    use super::*;

    #[test]
    fn version_names_round_trip_and_reject_anything_else() {
        assert_eq!(
            parse_version(&version_name(1_800_000_000)),
            Some(1_800_000_000)
        );
        for bad in [
            "browser.spark-backup.json",
            "browser.spark-backup..json",
            "browser.spark-backup.12a.json",
            "../browser.spark-backup.1.json",
            "browser.spark-backup.1.json/../x",
            "browser.spark-backup.1234567890123.json",
        ] {
            assert_eq!(parse_version(bad), None, "{bad}");
        }
        assert!(valid_revision_id("0B1x-_abc"));
        assert!(!valid_revision_id("a/b"));
        assert!(!valid_revision_id(""));
    }

    #[test]
    fn a_folder_keeps_the_newest_copies_only() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(SYNC_FILE_NAME);
        assert!(
            archive_folder(&file).is_ok(),
            "nothing to keep before the first send"
        );
        let versions = dir.path().join(DIR);
        std::fs::create_dir_all(&versions).unwrap();
        for time in 1..=7 {
            std::fs::write(versions.join(version_name(time)), b"{}").unwrap();
        }
        std::fs::write(versions.join("notes.txt"), b"other").unwrap();
        std::fs::write(&file, b"{}").unwrap();
        archive_folder(&file).unwrap();
        let kept = list_folder(&file).unwrap();
        assert_eq!(kept.len(), KEEP);
        assert!(
            kept.windows(2)
                .all(|pair| pair[0].created_at >= pair[1].created_at),
            "newest first"
        );
        assert!(
            versions.join("notes.txt").exists(),
            "other files are left alone"
        );
        assert!(read_folder_version(&file, "../browser.spark-backup.json", None).is_err());
    }

    #[test]
    fn propfind_answers_list_only_version_copies() {
        let xml = r#"<?xml version="1.0"?><D:multistatus xmlns:D="DAV:">
          <D:response><D:href>/dav/versions/</D:href><D:propstat><D:prop/></D:propstat></D:response>
          <D:response><D:href>/dav/versions/browser.spark-backup.1800000000.json</D:href><D:propstat><D:prop><D:getcontentlength>42</D:getcontentlength></D:prop></D:propstat></D:response>
          <D:response><D:href>/dav/versions/readme.txt</D:href></D:response>
        </D:multistatus>"#;
        assert_eq!(
            parse_propfind(xml),
            vec![SyncVersion {
                id: version_name(1_800_000_000),
                created_at: 1_800_000_000,
                size: Some(42)
            }]
        );
        let url = url::Url::parse("https://dav.example/dav/browser.spark-backup.json").unwrap();
        assert_eq!(
            webdav_versions_url(&url).unwrap().as_str(),
            "https://dav.example/dav/versions/"
        );
        assert!(webdav_version_url(&url, "../secret.json").is_err());
    }
}
