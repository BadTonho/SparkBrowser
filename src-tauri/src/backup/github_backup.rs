//! GitHub as a second, independent backup destination, next to the sync target in use (Google Drive,
//! a folder, Nextcloud or WebDAV). It has its own settings, token, last send and error, and sends a
//! full encrypted backup at most once an hour (the interface schedules it). Sending and reading go
//! through `github::Client`, which writes only to Spark's own branch of a private repository.
use super::{
    encryption, files, github,
    model::{BackupPreferences, RestoreGroup, RestoreMode, RestoreOptions},
    service,
    sync::{self, SyncPullResponse},
};
use crate::database::preferences_repo;
use crate::sync::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const CONFIG_KEY: &str = "backup_github_config_v1";
const LAST_AT_KEY: &str = "backup_github_last_at_v1";
const FINGERPRINT_KEY: &str = "backup_github_fingerprint_v1";

/// The repository GitHub backups go to.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitHubTarget {
    pub repository: String,
    pub branch: String,
    pub auto_save: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct GitHubBackupState {
    pub target: Option<GitHubTarget>,
    pub has_token: bool,
    pub last_synced_at: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Serialize)]
pub struct GitHubPushResult {
    pub synced: bool,
    pub skipped_unchanged: bool,
    pub state: GitHubBackupState,
}

/// The last failure, kept for this run like the sync target's.
#[derive(Default)]
pub struct GitHubBackupRuntime {
    last_error: Mutex<Option<String>>,
}

/// The token's place in the system credential store, apart from the sync target's password.
fn token_key(conn: &Connection) -> String {
    format!("{}/github", super::credentials::key(conn))
}

pub fn load_target(conn: &Connection) -> Result<Option<GitHubTarget>, String> {
    preferences_repo::load(conn, CONFIG_KEY)?
        .map(|raw| serde_json::from_str(&raw).map_err(|_| "backup.sync_invalid_config".to_string()))
        .transpose()
}

fn credentials(app: &AppHandle) -> std::sync::Arc<super::credentials::Credentials> {
    app.state::<service::BackupService>()
        .sync
        .credentials
        .clone()
}

pub fn state(app: &AppHandle, conn: &Connection) -> Result<GitHubBackupState, String> {
    let target = load_target(conn)?;
    let has_token = target.is_some() && credentials(app).read(&token_key(conn))?.is_some();
    Ok(GitHubBackupState {
        target,
        has_token,
        last_synced_at: preferences_repo::load(conn, LAST_AT_KEY)?
            .and_then(|value| value.parse().ok()),
        last_error: app
            .state::<GitHubBackupRuntime>()
            .last_error
            .lock()
            .map_err(|e| e.to_string())?
            .clone(),
    })
}

/// The saved token, when `repository` is the repository already saved (so it can be kept).
pub fn saved_token_for(
    app: &AppHandle,
    conn: &Connection,
    repository: &github::Repository,
) -> Result<Option<String>, String> {
    let same = load_target(conn)?.is_some_and(|target| {
        github::parse_repository(&target.repository).is_ok_and(|saved| saved == *repository)
    });
    if same {
        credentials(app).read(&token_key(conn))
    } else {
        Ok(None)
    }
}

/// Saves the repository and token (already checked on GitHub by the caller).
pub fn save(
    app: &AppHandle,
    conn: &Connection,
    target: GitHubTarget,
    token: &str,
) -> Result<GitHubBackupState, String> {
    let repository = github::parse_repository(&target.repository)?;
    github::validate_branch(&target.branch)?;
    if !encryption::enabled(conn)? {
        return Err("backup.github_encryption_required".into());
    }
    let changed = load_target(conn)?.is_none_or(|old| {
        github::parse_repository(&old.repository).ok() != Some(repository.clone())
            || old.branch != target.branch
    });
    credentials(app).write(&token_key(conn), &github::validate_token(token)?)?;
    let serialized = serde_json::to_string(&target).map_err(|e| e.to_string())?;
    preferences_repo::store(conn, CONFIG_KEY, &serialized)?;
    if changed {
        preferences_repo::remove(conn, FINGERPRINT_KEY)?;
        preferences_repo::remove(conn, LAST_AT_KEY)?;
    }
    set_error(app, None);
    state(app, conn)
}

pub fn set_auto(
    app: &AppHandle,
    conn: &Connection,
    auto_save: bool,
) -> Result<GitHubBackupState, String> {
    let mut target = load_target(conn)?.ok_or("backup.github_not_connected")?;
    target.auto_save = auto_save;
    let serialized = serde_json::to_string(&target).map_err(|e| e.to_string())?;
    preferences_repo::store(conn, CONFIG_KEY, &serialized)?;
    state(app, conn)
}

/// Forgets the repository and the token on this computer (the token stays valid on GitHub).
pub fn disconnect(app: &AppHandle, conn: &Connection) -> Result<GitHubBackupState, String> {
    credentials(app).delete(&token_key(conn))?;
    for key in [CONFIG_KEY, LAST_AT_KEY, FINGERPRINT_KEY] {
        preferences_repo::remove(conn, key)?;
    }
    set_error(app, None);
    state(app, conn)
}

fn set_error(app: &AppHandle, error: Option<String>) {
    if let Ok(mut last) = app.state::<GitHubBackupRuntime>().last_error.lock() {
        *last = error;
    }
}

fn with_db<T>(
    app: &AppHandle,
    work: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let state = app.state::<crate::state::browser_state::BrowserState>();
    let db = state
        .infrastructure()
        .db()
        .lock()
        .map_err(|e| e.to_string())?;
    work(db.as_ref().ok_or("backup.database_unavailable")?)
}

/// The client of the saved repository, its branch and the read password.
fn client(app: &AppHandle) -> Result<(github::Client, String, Option<String>), String> {
    with_db(app, |conn| {
        let target = load_target(conn)?.ok_or("backup.github_not_connected")?;
        let token = credentials(app)
            .read(&token_key(conn))?
            .ok_or("backup.github_token_missing")?;
        let password = encryption::read_password(conn, &credentials(app));
        Ok((
            github::Client::new(&token, github::parse_repository(&target.repository)?)?,
            target.branch,
            password,
        ))
    })
}

/// Sends a full backup. With `only_if_changed`, nothing is sent when the data did not change since
/// the last send.
pub async fn push(
    app: &AppHandle,
    preferences: BackupPreferences,
    only_if_changed: bool,
) -> Result<GitHubPushResult, String> {
    let service = app.state::<service::BackupService>();
    let _serial = service.operation.lock().await;
    let document = service::capture(app, preferences).await?;
    let fingerprint = sync::fingerprint_hex(&document)?;
    let (previous, password) = with_db(app, |conn| {
        if load_target(conn)?.is_none() {
            return Err("backup.github_not_connected".to_string());
        }
        Ok((
            preferences_repo::load(conn, FINGERPRINT_KEY)?,
            encryption::write_password(conn, &service.sync.credentials)?,
        ))
    })?;
    if only_if_changed && previous.as_deref() == Some(fingerprint.as_str()) {
        let state = with_db(app, |conn| state(app, conn))?;
        return Ok(GitHubPushResult {
            synced: false,
            skipped_unchanged: true,
            state,
        });
    }
    let keep_version = previous.as_deref() != Some(fingerprint.as_str());
    let outcome = async {
        let password = password.ok_or("backup.github_encryption_required")?;
        let (client, branch, _) = client(app)?;
        tauri::async_runtime::spawn_blocking(move || {
            let bytes = files::encode(&document, Some(&password))?;
            client.push(&branch, &bytes, keep_version)
        })
        .await
        .map_err(|e| e.to_string())?
    }
    .await;
    if let Err(error) = outcome {
        set_error(app, Some(error.clone()));
        return Err(error);
    }
    set_error(app, None);
    let now = crate::state::tab_model::TabInfo::current_epoch_secs() as i64;
    let state = with_db(app, |conn| {
        preferences_repo::store(conn, FINGERPRINT_KEY, &fingerprint)?;
        preferences_repo::store(conn, LAST_AT_KEY, &now.to_string())?;
        state(app, conn)
    })?;
    Ok(GitHubPushResult {
        synced: true,
        skipped_unchanged: false,
        state,
    })
}

/// Opens a document from GitHub in the review panel; nothing changes until it is confirmed there.
async fn review_document(
    app: &AppHandle,
    document: Result<super::model::BackupDocument, String>,
    current_preferences: BackupPreferences,
) -> Result<SyncPullResponse, String> {
    let document = document.inspect_err(|error| set_error(app, Some(error.clone())))?;
    let backup_service = app.state::<service::BackupService>();
    let source_token = backup_service.select(document)?;
    let preview = service::preview_with(
        app,
        None,
        Some(source_token.clone()),
        current_preferences,
        RestoreOptions {
            groups: RestoreGroup::ALL.to_vec(),
            mode: RestoreMode::Merge,
        },
    )
    .await?;
    let state = with_db(app, |conn| sync::snapshot(conn, &backup_service.sync))?;
    Ok(SyncPullResponse::Preview {
        source_token,
        preview: Box::new(preview),
        state,
    })
}

pub async fn review(
    app: &AppHandle,
    current_preferences: BackupPreferences,
) -> Result<SyncPullResponse, String> {
    let (client, branch, password) = client(app)?;
    let document =
        tauri::async_runtime::spawn_blocking(move || client.pull(&branch, password.as_deref()))
            .await
            .map_err(|e| e.to_string())?;
    review_document(app, document, current_preferences).await
}

pub async fn list_versions(
    app: &AppHandle,
) -> Result<Vec<super::sync_versions::SyncVersion>, String> {
    let (client, branch, _) = client(app)?;
    tauri::async_runtime::spawn_blocking(move || client.list_versions(&branch))
        .await
        .map_err(|e| e.to_string())?
}

pub async fn review_version(
    app: &AppHandle,
    id: String,
    current_preferences: BackupPreferences,
) -> Result<SyncPullResponse, String> {
    let (client, branch, password) = client(app)?;
    let document = tauri::async_runtime::spawn_blocking(move || {
        client.read_version(&branch, &id, password.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?;
    review_document(app, document, current_preferences).await
}
