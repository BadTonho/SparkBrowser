use super::{
    files,
    manager::{Control, DownloadManager},
    model::DownloadRecord,
};
use sha2::{Digest, Sha256};
use std::{
    sync::{atomic::Ordering, OnceLock},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn empty_hash() -> String {
    format!("{:x}", Sha256::new().finalize())
}
fn stopped(control: &Control) -> Result<(), String> {
    if control.requested.load(Ordering::Acquire) != 0 {
        Err("downloads.stopped".into())
    } else {
        Ok(())
    }
}
/// Many video servers refuse requests without a page and a browser name.
const VIDEO_USER_AGENT: &str = "Mozilla/5.0 (compatible; Spark)";

/// Sends the page a video came from the way a browser does: another origin only learns its origin.
fn referer_value(page: &str, target: &tauri::Url) -> Option<String> {
    let mut page = tauri::Url::parse(page).ok()?;
    page.set_fragment(None);
    if page.origin() == target.origin() {
        return Some(page.to_string());
    }
    if page.scheme() == "https" && target.scheme() != "https" {
        return None;
    }
    Some(format!("{}/", page.origin().ascii_serialization()))
}
/// Only video downloads carry a page; every other request stays as it was.
pub(super) fn with_referer(
    request: reqwest::RequestBuilder,
    target: &tauri::Url,
    page: Option<&str>,
) -> reqwest::RequestBuilder {
    let Some(page) = page else {
        return request;
    };
    let request = request.header(reqwest::header::USER_AGENT, VIDEO_USER_AGENT);
    match referer_value(page, target) {
        Some(value) => request.header(reqwest::header::REFERER, value),
        None => request,
    }
}
pub(super) fn client(shield: bool) -> Result<&'static reqwest::Client, String> {
    static ON: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    static OFF: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    (if shield { &ON } else { &OFF })
        .get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(20))
                .read_timeout(Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::custom(move |a| {
                    if a.previous().len() >= 10 {
                        a.error("downloads.redirect_limit")
                    } else if super::transfer::validate_download_url(a.url().as_str()).is_err()
                        || crate::privacy::adblock_engine::should_block_url(
                            a.url().as_str(),
                            shield,
                        )
                        || a.previous().first().is_some_and(|first| {
                            !super::local_network::redirect_allowed(first, a.url())
                        })
                    {
                        a.error("downloads.blocked")
                    } else {
                        a.follow()
                    }
                }))
                .build()
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)
}
pub fn range(value: &str, offset: u64) -> Option<u64> {
    let (span, total) = value.strip_prefix("bytes ")?.split_once('/')?;
    let (start, end) = span.split_once('-')?;
    let start = start.parse::<u64>().ok()?;
    let end = end.parse::<u64>().ok()?;
    let total = total.parse::<u64>().ok()?;
    (start == offset && end.checked_add(1) == Some(total) && end >= start).then_some(total)
}
async fn hash_file(
    app: &AppHandle,
    id: &str,
    path: &std::path::Path,
    limit: Option<u64>,
    control: &Control,
) -> Result<(u64, Sha256), String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|_| "downloads.file_missing")?;
    let mut hash = Sha256::new();
    let mut count = 0;
    let mut buffer = [0; 65536];
    let mut last = Instant::now();
    loop {
        stopped(control)?;
        let max = (limit.unwrap_or(u64::MAX) - count).min(buffer.len() as u64) as usize;
        if max == 0 {
            break;
        }
        let n = tokio::select! {biased;_=control.changed.notified()=>return Err("downloads.stopped".into()),r=file.read(&mut buffer[..max])=>r.map_err(|e|e.to_string())?};
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
        count += n as u64;
        if last.elapsed() > Duration::from_millis(100) {
            app.state::<DownloadManager>()
                .update(app, id, |t| t.verification_received = count);
            last = Instant::now();
        }
    }
    Ok((count, hash))
}
async fn checkpoint(
    app: &AppHandle,
    id: &str,
    file: &tokio::fs::File,
    count: u64,
    hash: &Sha256,
) -> Result<(), String> {
    file.sync_all().await.map_err(|e| e.to_string())?;
    app.state::<DownloadManager>().edit(app, id, true, |r| {
        r.confirmed = count;
        r.prefix_sha256 = format!("{:x}", hash.clone().finalize());
        r.task.received = count;
    })?;
    Ok(())
}
pub(super) async fn finalize(
    app: &AppHandle,
    id: &str,
    record: &DownloadRecord,
    count: u64,
    hash: Sha256,
) -> Result<(), String> {
    let actual = format!("{:x}", hash.finalize());
    let manager = app.state::<DownloadManager>();
    manager.edit(app, id, true, |r| {
        r.body_completed = true;
        r.confirmed = count;
        r.prefix_sha256 = actual.clone();
        r.task.actual_sha256 = Some(actual.clone());
        r.task.received = count;
        r.task.total = Some(count);
        r.task.verification_received = count;
    })?;
    if record
        .task
        .expected_sha256
        .as_ref()
        .is_some_and(|expected| expected != &actual)
    {
        return Err("downloads.checksum_mismatch".into());
    }
    manager.update_saved(app, id, |t| t.status = "Finalizing".into())?;
    manager.complete(app, id)
}
pub(super) fn shield_for(app: &AppHandle, record: &DownloadRecord) -> Result<bool, String> {
    let state = app.state::<crate::state::browser_state::BrowserState>();
    Ok(match record.task.source_page_url.as_deref() {
        Some(source) => {
            *state
                .policies()
                .shield_enabled()
                .lock()
                .map_err(|e| e.to_string())?
                && crate::privacy::adblock_engine::site_is_protected(&state, source)
                && crate::privacy::adblock_engine::site_is_protected(&state, &record.task.url)
        }
        None => {
            crate::privacy::adblock_engine::download_shield_enabled(&state, None, &record.task.url)
        }
    })
}
pub async fn run(app: &AppHandle, id: &str, control: &Control, verify: bool) -> Result<(), String> {
    stopped(control)?;
    let manager = app.state::<DownloadManager>();
    let record = manager.record(id)?;
    if verify {
        let partial = record.body_completed && record.partial().is_file();
        let path = if partial {
            files::owned_partial(&record)?
        } else {
            record
                .task
                .save_path
                .as_ref()
                .ok_or("downloads.file_missing")?
                .into()
        };
        let (count, hash) = hash_file(app, id, &path, None, control).await?;
        if partial {
            return finalize(app, id, &record, count, hash).await;
        }
        let actual = format!("{:x}", hash.finalize());
        manager.update_saved(app, id, |t| {
            t.actual_sha256 = Some(actual.clone());
            t.verification_received = count;
            t.status = "Completed".into();
        })?;
        if record
            .task
            .expected_sha256
            .as_ref()
            .is_some_and(|expected| expected != &actual)
        {
            return Err("downloads.checksum_mismatch".into());
        }
        return Ok(());
    }
    let path = files::owned_partial(&record)?;
    let mut hash = Sha256::new();
    if record.confirmed > 0 {
        let (count, prefix) = hash_file(app, id, &path, Some(record.confirmed), control).await?;
        if count != record.confirmed
            || format!("{:x}", prefix.clone().finalize()) != record.prefix_sha256
        {
            return Err("downloads.file_changed".into());
        }
        hash = prefix;
    }
    stopped(control)?;
    let shield = shield_for(app, &record)?;
    if crate::privacy::adblock_engine::should_block_url(&record.task.url, shield) {
        return Err("downloads.blocked".into());
    }
    let mut request = client(shield)?
        .get(&record.task.url)
        .header(reqwest::header::ACCEPT_ENCODING, "identity");
    if let Ok(target) = tauri::Url::parse(&record.task.url) {
        request = with_referer(request, &target, record.referer.as_deref());
    }
    if record.confirmed > 0 {
        request = request
            .header(
                reqwest::header::RANGE,
                format!("bytes={}-", record.confirmed),
            )
            .header(
                reqwest::header::IF_RANGE,
                record.etag.as_ref().ok_or("downloads.restart_required")?,
            );
    }
    let mut response = tokio::select! {biased;_=control.changed.notified()=>return Err("downloads.stopped".into()),r=request.send()=>r.map_err(|_|"downloads.network_error")?};
    let final_url = response.url().to_string();
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.starts_with('"') && v.ends_with('"'))
        .map(str::to_string);
    let total = if record.confirmed > 0 {
        if response.status() != reqwest::StatusCode::PARTIAL_CONTENT
            || record.final_url.as_ref() != Some(&final_url)
            || etag != record.etag
        {
            return Err("downloads.restart_required".into());
        }
        let total = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| range(v, record.confirmed))
            .ok_or("downloads.restart_required")?;
        if record.task.total.is_some_and(|v| v != total)
            || response
                .content_length()
                .is_some_and(|v| v != total - record.confirmed)
        {
            return Err("downloads.restart_required".into());
        }
        Some(total)
    } else {
        if response.status() != reqwest::StatusCode::OK {
            return Err("downloads.http_error".into());
        }
        response.content_length()
    };
    if response
        .headers()
        .get(reqwest::header::CONTENT_ENCODING)
        .is_some_and(|v| v != "identity")
    {
        return Err("downloads.restart_required".into());
    }
    manager.edit(app, id, true, |r| {
        r.final_url = Some(final_url);
        r.etag = etag;
        r.task.total = total;
        r.task.status = "Downloading".into();
    })?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .await
        .map_err(|e| e.to_string())?;
    file.set_len(record.confirmed)
        .await
        .map_err(|e| e.to_string())?;
    use tokio::io::AsyncSeekExt;
    file.seek(std::io::SeekFrom::Start(record.confirmed))
        .await
        .map_err(|e| e.to_string())?;
    let mut count = record.confirmed;
    let mut last_checkpoint = Instant::now();
    let mut checkpoint_bytes = count;
    let mut progress = Instant::now();
    let result:Result<(),String>=async {
        loop{stopped(control)?;
            let chunk=tokio::select!{biased;_=control.changed.notified()=>return Err("downloads.stopped".into()),r=response.chunk()=>r.map_err(|_|"downloads.network_error")?};
            let Some(chunk)=chunk else{break;};
            if total.is_some_and(|n|count+chunk.len() as u64>n){return Err("downloads.restart_required".into());}
            file.write_all(&chunk).await.map_err(|e|e.to_string())?;hash.update(&chunk);count+=chunk.len() as u64;
            if progress.elapsed()>Duration::from_millis(100){manager.update(app,id,|t|t.received=count);progress=Instant::now();}
            if last_checkpoint.elapsed()>Duration::from_secs(1) || count-checkpoint_bytes>=1024*1024{checkpoint(app,id,&file,count,&hash).await?;last_checkpoint=Instant::now();checkpoint_bytes=count;}
        }
        if total.is_some_and(|n|n!=count){return Err("downloads.network_error".into());}Ok(())
    }.await;
    checkpoint(app, id, &file, count, &hash).await?;
    drop(file);
    result?;
    stopped(control)?;
    finalize(app, id, &record, count, hash).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_referer_never_leaks_the_page_path_to_other_origins() {
        let page = "https://page.example.test/watch?private=1#t";
        let value = |target: &str| referer_value(page, &tauri::Url::parse(target).unwrap());
        assert_eq!(
            value("https://page.example.test/v.mp4").as_deref(),
            Some("https://page.example.test/watch?private=1")
        );
        assert_eq!(
            value("https://cdn.example.test/v.mp4").as_deref(),
            Some("https://page.example.test/")
        );
        assert_eq!(value("http://cdn.example.test/v.mp4"), None);
    }
    #[test]
    fn validates_exact_remaining_ranges() {
        assert_eq!(range("bytes 3-9/10", 3), Some(10));
        for value in [
            "bytes 2-9/10",
            "bytes 3-8/10",
            "bytes 3-9/*",
            "bytes 9-3/10",
            "bytes */10",
        ] {
            assert_eq!(range(value, 3), None);
        }
    }
}
