use super::{
    http::{CheckError, GitHub},
    model::{UpdateChannel, UpdateInfo, UpdateState},
    releases, UpdateService,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

pub async fn find(
    app: &AppHandle,
    version: &str,
    channel: UpdateChannel,
) -> Result<Option<(Update, UpdateInfo)>, CheckError> {
    let current = semver::Version::parse(version).map_err(CheckError::plain)?;
    let github = GitHub::new(version)?;
    let candidates = releases::candidates(github.releases().await?, &current, channel);
    for release in candidates {
        let url = releases::manifest_url(&release).map_err(CheckError::plain)?;
        let manifest = github.json(&url).await?;
        if !releases::validate_manifest(&release, &manifest).map_err(CheckError::plain)? {
            continue;
        }
        let app_for_exit = app.clone();
        let updater = app
            .updater_builder()
            .endpoints(vec![url.parse().map_err(CheckError::plain)?])
            .map_err(CheckError::plain)?
            .target(releases::TARGET)
            .on_before_exit(move || {
                // Persistence was frozen/drained and the session committed before install().
                app_for_exit
                    .state::<crate::downloads::manager::DownloadManager>()
                    .shutdown();
                app_for_exit.cleanup_before_exit();
            })
            .build()
            .map_err(CheckError::plain)?;
        let mut update = updater
            .restore_update(manifest)
            .map_err(CheckError::plain)?
            .ok_or_else(|| {
                CheckError::plain("Selected release is not newer than this application")
            })?;
        update.timeout = Some(Duration::from_secs(300));
        return Ok(Some((update, releases::info(&release))));
    }
    Ok(None)
}

pub async fn download(app: &AppHandle, expected_version: &str) -> Result<UpdateState, String> {
    let service = app.state::<UpdateService>();
    let _operation = service
        .operation
        .try_lock()
        .map_err(|_| "An update operation is already running")?;
    let changed = service.changed.notified();
    tokio::pin!(changed);
    changed.as_mut().enable();
    let (candidate, generation) = {
        let mut inner = service.inner.lock().map_err(|e| e.to_string())?;
        if !inner.snapshot.supported {
            return Err(inner.snapshot.message.clone());
        }
        let candidate = inner
            .candidate
            .clone()
            .filter(|c| c.version == expected_version)
            .ok_or("The selected update changed. Check again.")?;
        if inner.snapshot.status == "ready" {
            return Ok(inner.snapshot.clone());
        }
        inner.snapshot.status = "downloading".into();
        inner.snapshot.error = None;
        inner.snapshot.received = 0;
        inner.snapshot.total = None;
        (candidate, inner.generation)
    };
    service.publish(app)?;
    let mut last_progress = Instant::now();
    let download = candidate.download(
        |size, total| {
            if let Ok(mut inner) = service.inner.lock() {
                if inner.generation != generation {
                    return;
                }
                inner.snapshot.received = inner.snapshot.received.saturating_add(size as u64);
                inner.snapshot.total = total;
            }
            if last_progress.elapsed() >= Duration::from_millis(100) {
                let _ = service.publish(app);
                last_progress = Instant::now();
            }
        },
        || {},
    );
    let result = tokio::select! {
        result = download => result,
        _ = changed => return service.snapshot(),
    };
    let mut to_stage = None;
    {
        let mut inner = service.inner.lock().map_err(|e| e.to_string())?;
        if inner.generation != generation {
            return Ok(inner.snapshot.clone());
        }
        match result {
            Ok(bytes) => {
                let bytes = Arc::new(bytes);
                if inner.snapshot.auto_update {
                    to_stage = Some((
                        super::staged::Staged {
                            version: candidate.version.clone(),
                            channel: inner.snapshot.channel.as_str().into(),
                            manifest: candidate.raw_json.clone(),
                        },
                        bytes.clone(),
                    ));
                }
                inner.bytes = Some(bytes);
                inner.snapshot.status = "ready".into();
            }
            Err(_) => {
                inner.bytes = None;
                inner.snapshot.status = "error".into();
                inner.snapshot.error = Some("Download or signature verification failed. No update was installed. Try downloading again.".into());
            }
        }
    }
    // The verified installer is also kept on disk, so that an update nobody restarted for installs the next
    // time Spark opens. A failure here leaves the in-memory update usable with the restart button.
    if let Some((staged, bytes)) = to_stage {
        let directory = super::staged::directory(app)?;
        let saved = tauri::async_runtime::spawn_blocking(move || {
            super::staged::write(&directory, &staged, bytes.as_slice())
        })
        .await
        .map_err(|e| e.to_string())
        .and_then(|saved| saved);
        let mut inner = service.inner.lock().map_err(|e| e.to_string())?;
        if inner.generation == generation {
            match saved {
                Ok(()) => inner.snapshot.staged = true,
                Err(error) => {
                    inner.snapshot.error = Some(format!(
                        "The update could not be saved for the next start: {error}"
                    ))
                }
            }
        }
    }
    service.publish(app)
}

pub async fn install(
    app: AppHandle,
    expected_version: String,
    _confirmed_activity: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let service = app.state::<UpdateService>();
        let _operation = service
            .operation
            .try_lock()
            .map_err(|_| "An update operation is already running")?;
        let (candidate, bytes) = {
            let mut inner = service.inner.lock().map_err(|e| e.to_string())?;
            if !inner.snapshot.supported || inner.snapshot.status != "ready" {
                return Err("Download and verify the update first".to_owned());
            }
            let candidate = inner
                .candidate
                .clone()
                .filter(|c| c.version == expected_version)
                .ok_or("The selected update changed. Check again.")?;
            let bytes = inner
                .bytes
                .clone()
                .ok_or("The downloaded update is no longer available")?;
            candidate
                .verify(&bytes)
                .map_err(|_| "Update signature verification failed")?;
            inner.snapshot.status = "installing".into();
            inner.snapshot.error = None;
            (candidate, bytes)
        };
        service.publish(&app)?;
        // The restart installs it now; a copy kept for the next start would only run it twice.
        super::staged::clear_for(&app);
        // The shared shutdown coordinator already froze writers and committed the clean marker.
        let result = candidate
            .install(bytes.as_slice())
            .map_err(|e| format!("Could not start the update installer: {e}"));
        if let Err(error) = &result {
            let mut inner = service.inner.lock().map_err(|e| e.to_string())?;
            inner.snapshot.status = "ready".into();
            inner.snapshot.error = Some(error.clone());
        }
        service.publish(&app)?;
        result
    })
    .await
    .map_err(|e| e.to_string())?
}
