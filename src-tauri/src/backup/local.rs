//! Local backup history: a bounded set of timestamped snapshots in the app data
//! directory, produced manually, on a schedule or when the browser closes.
//! Restoring one reuses the reviewed-import pipeline (select → preview → commit).
use super::{crypto, encryption, files, model::BackupPreferences, service, sync};
use crate::database::preferences_repo as prefs;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

const DIR_NAME: &str = "backups";
const PREFIX: &str = "backup-";
const SUFFIX: &str = ".veloce-backup.json";
const SNIFF_BYTES: usize = 160;
pub const DEFAULT_MAX_BACKUPS: u32 = 10;
pub const MAX_BACKUPS_LIMIT: u32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Manual,
    Scheduled,
    Exit,
}
impl Reason {
    fn as_str(self) -> &'static str {
        match self {
            Reason::Manual => "manual",
            Reason::Scheduled => "scheduled",
            Reason::Exit => "exit",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        match value {
            "manual" => Some(Reason::Manual),
            "scheduled" => Some(Reason::Scheduled),
            "exit" => Some(Reason::Exit),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interval {
    SixHours,
    Daily,
    Weekly,
}
impl Interval {
    pub fn seconds(self) -> i64 {
        match self {
            Interval::SixHours => 6 * 3600,
            Interval::Daily => 24 * 3600,
            Interval::Weekly => 7 * 24 * 3600,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    pub enabled: bool,
    pub interval: Interval,
    pub on_exit: bool,
    pub max_backups: u32,
}
impl Default for Schedule {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: Interval::Daily,
            on_exit: false,
            max_backups: DEFAULT_MAX_BACKUPS,
        }
    }
}
impl Schedule {
    fn validate(&self) -> Result<(), String> {
        if (1..=MAX_BACKUPS_LIMIT).contains(&self.max_backups) {
            Ok(())
        } else {
            Err("backup.schedule_invalid".into())
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    pub id: String,
    pub created_at: i64,
    pub reason: Reason,
    pub size_bytes: u64,
    pub encrypted: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LocalState {
    pub schedule: Schedule,
    pub encryption_enabled: bool,
    pub last_run_at: Option<i64>,
    /// Epoch seconds at which the next scheduled backup is due; `None` when disabled.
    pub next_due_at: Option<i64>,
    pub directory: String,
    pub entries: Vec<Entry>,
    pub total_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOutcome {
    pub created: bool,
    pub state: LocalState,
}

fn now() -> i64 {
    crate::state::tab_model::TabInfo::current_epoch_secs() as i64
}

fn directory(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(DIR_NAME);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn file_name(created_at: i64, reason: Reason) -> String {
    format!("{PREFIX}{created_at}-{}{SUFFIX}", reason.as_str())
}

/// Only names this module generates are accepted, which also rules out path traversal.
fn parse_name(name: &str) -> Option<(i64, Reason)> {
    let rest = name.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    let (epoch, reason) = rest.split_once('-')?;
    if epoch.is_empty() || epoch.len() > 12 || !epoch.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((epoch.parse().ok()?, Reason::parse(reason)?))
}

fn path_for_id(dir: &Path, id: &str) -> Result<PathBuf, String> {
    parse_name(id).ok_or("backup.history_not_found")?;
    let path = dir.join(id);
    if path.is_file() {
        Ok(path)
    } else {
        Err("backup.history_not_found".into())
    }
}

fn looks_encrypted(path: &Path) -> bool {
    let mut head = Vec::with_capacity(SNIFF_BYTES);
    fs::File::open(path)
        .and_then(|file| file.take(SNIFF_BYTES as u64).read_to_end(&mut head))
        .is_ok()
        && String::from_utf8_lossy(&head).contains(crypto::ENCRYPTED_FORMAT)
}

/// Newest first. Foreign files and temporary exports are ignored.
fn list(dir: &Path) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    for item in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let Some(name) = item.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some((created_at, reason)) = parse_name(&name) else {
            continue;
        };
        let Ok(meta) = item.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        entries.push(Entry {
            encrypted: looks_encrypted(&item.path()),
            id: name,
            created_at,
            reason,
            size_bytes: meta.len(),
        });
    }
    entries.sort_by(|a, b| (b.created_at, &b.id).cmp(&(a.created_at, &a.id)));
    Ok(entries)
}

/// Keeps the newest `keep` snapshots and returns how many were removed.
fn prune(dir: &Path, keep: u32) -> Result<usize, String> {
    let mut removed = 0;
    for entry in list(dir)?.into_iter().skip(keep as usize) {
        if fs::remove_file(dir.join(&entry.id)).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

/// Writes atomically under a name that never replaces an existing snapshot.
fn write_snapshot(dir: &Path, created_at: i64, reason: Reason, bytes: &[u8]) -> Result<(), String> {
    for offset in 0..60 {
        let path = dir.join(file_name(created_at + offset, reason));
        if !path.exists() {
            return files::write_bytes(&path, bytes);
        }
    }
    Err("backup.invalid_path".into())
}

pub fn load_schedule(conn: &Connection) -> Schedule {
    prefs::load(conn, prefs::BACKUP_LOCAL_SCHEDULE_KEY)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Schedule>(&raw).ok())
        .filter(|schedule| schedule.validate().is_ok())
        .unwrap_or_default()
}

fn build_state(conn: &Connection, dir: &Path) -> Result<LocalState, String> {
    let schedule = load_schedule(conn);
    let last_run_at = prefs::load(conn, prefs::BACKUP_LOCAL_LAST_RUN_KEY)?
        .and_then(|value| value.parse::<i64>().ok());
    let next_due_at = schedule
        .enabled
        .then(|| last_run_at.map_or_else(now, |last| last + schedule.interval.seconds()));
    let entries = list(dir)?;
    Ok(LocalState {
        schedule,
        encryption_enabled: encryption::enabled(conn)?,
        last_run_at,
        next_due_at,
        directory: dir.to_string_lossy().into_owned(),
        total_bytes: entries.iter().map(|entry| entry.size_bytes).sum(),
        entries,
    })
}

pub async fn state(app: &AppHandle) -> Result<LocalState, String> {
    let dir = directory(app)?;
    crate::database::executor::run(app.clone(), move |conn| build_state(conn, &dir)).await
}

pub async fn set_schedule(app: &AppHandle, schedule: Schedule) -> Result<LocalState, String> {
    schedule.validate()?;
    let dir = directory(app)?;
    crate::database::executor::run(app.clone(), move |conn| {
        let raw = serde_json::to_string(&schedule).map_err(|e| e.to_string())?;
        prefs::store(conn, prefs::BACKUP_LOCAL_SCHEDULE_KEY, &raw)?;
        // A lowered limit applies immediately instead of at the next backup.
        prune(&dir, schedule.max_backups)?;
        build_state(conn, &dir)
    })
    .await
}

pub async fn set_encryption(
    app: &AppHandle,
    enable: bool,
    password: Option<String>,
) -> Result<LocalState, String> {
    let dir = directory(app)?;
    let worker = app.clone();
    crate::database::executor::run(app.clone(), move |conn| {
        let service = worker.state::<service::BackupService>();
        encryption::configure(conn, &service.sync.credentials, enable, password)?;
        build_state(conn, &dir)
    })
    .await
}

/// Captures and stores a snapshot. With `only_if_changed`, an unchanged document
/// only refreshes the schedule clock instead of piling up identical files.
pub async fn create(
    app: &AppHandle,
    preferences: BackupPreferences,
    reason: Reason,
    only_if_changed: bool,
) -> Result<CreateOutcome, String> {
    let backup = app.state::<service::BackupService>();
    let _serial = backup.operation.lock().await;
    let document = service::capture(app, preferences).await?;
    let fingerprint = sync::fingerprint_hex(&document)?;
    let dir = directory(app)?;

    let worker = app.clone();
    let (skip, password, max_backups) = {
        let (dir, fingerprint) = (dir.clone(), fingerprint.clone());
        crate::database::executor::run(app.clone(), move |conn| {
            let schedule = load_schedule(conn);
            let unchanged = only_if_changed
                && prefs::load(conn, prefs::BACKUP_LOCAL_FINGERPRINT_KEY)?.as_deref()
                    == Some(fingerprint.as_str())
                && !list(&dir)?.is_empty();
            if unchanged {
                prefs::store(conn, prefs::BACKUP_LOCAL_LAST_RUN_KEY, &now().to_string())?;
                return Ok((true, None, schedule.max_backups));
            }
            let credentials = &worker.state::<service::BackupService>().sync.credentials;
            let password = encryption::write_password(conn, credentials)?;
            Ok((false, password, schedule.max_backups))
        })
        .await?
    };

    if !skip {
        let dir = dir.clone();
        tauri::async_runtime::spawn_blocking(move || {
            // Key derivation and fsync stay off the database lock.
            let bytes = files::encode(&document, password.as_deref())?;
            write_snapshot(&dir, now(), reason, &bytes)?;
            prune(&dir, max_backups).map(|_| ())
        })
        .await
        .map_err(|e| e.to_string())??;
    }

    let dir_for_state = dir;
    crate::database::executor::run(app.clone(), move |conn| {
        if !skip {
            prefs::store(conn, prefs::BACKUP_LOCAL_FINGERPRINT_KEY, &fingerprint)?;
            prefs::store(conn, prefs::BACKUP_LOCAL_LAST_RUN_KEY, &now().to_string())?;
        }
        Ok(CreateOutcome {
            created: !skip,
            state: build_state(conn, &dir_for_state)?,
        })
    })
    .await
}

pub async fn delete(app: &AppHandle, id: String) -> Result<LocalState, String> {
    let dir = directory(app)?;
    crate::database::executor::run(app.clone(), move |conn| {
        fs::remove_file(path_for_id(&dir, &id)?).map_err(|e| e.to_string())?;
        build_state(conn, &dir)
    })
    .await
}

/// Loads a snapshot into the import pipeline and returns its selection token.
/// A missing or wrong stored password keeps the file locked for `unlock`.
pub async fn restore(app: &AppHandle, id: String) -> Result<String, String> {
    let backup = app.state::<service::BackupService>();
    let _serial = backup.operation.lock().await;
    let dir = directory(app)?;
    let worker = app.clone();
    let (bytes, password) = crate::database::executor::run(app.clone(), move |conn| {
        let path = path_for_id(&dir, &id)?;
        let credentials = &worker.state::<service::BackupService>().sync.credentials;
        Ok((
            files::read_bytes(&path)?,
            encryption::read_password(conn, credentials),
        ))
    })
    .await?;
    let (decoded, bytes) = tauri::async_runtime::spawn_blocking(move || {
        (files::decode_with(&bytes, password.as_deref()), bytes)
    })
    .await
    .map_err(|e| e.to_string())?;
    match decoded {
        Ok(document) => backup.select(document),
        Err(error) if error == "backup.password_required" || error == "backup.password_wrong" => {
            backup.hold_locked(bytes)?;
            Err(error)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn names_roundtrip_and_reject_traversal_or_foreign_files() {
        let name = file_name(1_700_000_000, Reason::Scheduled);
        assert_eq!(parse_name(&name), Some((1_700_000_000, Reason::Scheduled)));
        for bad in [
            "../backup-1-manual.veloce-backup.json",
            "backup-1-manual.veloce-backup.json.bak",
            "backup--manual.veloce-backup.json",
            "backup-1-other.veloce-backup.json",
            "backup-1.veloce-backup.json",
            "notes.txt",
            "backup-1234567890123-manual.veloce-backup.json",
        ] {
            assert_eq!(parse_name(bad), None, "{bad}");
        }
        let dir = temp();
        assert_eq!(
            path_for_id(dir.path(), "../x").unwrap_err(),
            "backup.history_not_found"
        );
        assert_eq!(
            path_for_id(dir.path(), &name).unwrap_err(),
            "backup.history_not_found"
        );
    }

    #[test]
    fn list_orders_newest_first_flags_encryption_and_ignores_foreign_files() {
        let dir = temp();
        write_snapshot(
            dir.path(),
            100,
            Reason::Manual,
            b"{\n  \"format\": \"veloce-backup\"\n}",
        )
        .unwrap();
        write_snapshot(
            dir.path(),
            300,
            Reason::Exit,
            b"{\n  \"format\": \"veloce-backup-encrypted\"\n}",
        )
        .unwrap();
        write_snapshot(dir.path(), 200, Reason::Scheduled, b"{}").unwrap();
        fs::write(dir.path().join("readme.txt"), "x").unwrap();
        fs::write(dir.path().join(".veloce-backup-abc.tmp"), "x").unwrap();
        let entries = list(dir.path()).unwrap();
        assert_eq!(
            entries.iter().map(|e| e.created_at).collect::<Vec<_>>(),
            [300, 200, 100]
        );
        assert_eq!(
            entries.iter().map(|e| e.encrypted).collect::<Vec<_>>(),
            [true, false, false]
        );
        assert_eq!(entries[0].reason, Reason::Exit);
    }

    #[test]
    fn same_second_snapshots_never_overwrite_each_other() {
        let dir = temp();
        for _ in 0..3 {
            write_snapshot(dir.path(), 50, Reason::Manual, b"{}").unwrap();
        }
        assert_eq!(list(dir.path()).unwrap().len(), 3);
    }

    #[test]
    fn prune_keeps_only_the_newest_snapshots() {
        let dir = temp();
        for at in 1..=5 {
            write_snapshot(dir.path(), at * 10, Reason::Scheduled, b"{}").unwrap();
        }
        assert_eq!(prune(dir.path(), 2).unwrap(), 3);
        assert_eq!(
            list(dir.path())
                .unwrap()
                .iter()
                .map(|e| e.created_at)
                .collect::<Vec<_>>(),
            [50, 40]
        );
        assert_eq!(prune(dir.path(), 2).unwrap(), 0);
    }

    #[test]
    fn schedule_is_validated_persisted_and_falls_back_to_defaults() {
        let conn = Connection::open_in_memory().unwrap();
        prefs::initialize_preferences_table(&conn).unwrap();
        assert_eq!(load_schedule(&conn), Schedule::default());
        let valid = Schedule {
            enabled: true,
            interval: Interval::Weekly,
            on_exit: true,
            max_backups: 25,
        };
        prefs::store(
            &conn,
            prefs::BACKUP_LOCAL_SCHEDULE_KEY,
            &serde_json::to_string(&valid).unwrap(),
        )
        .unwrap();
        assert_eq!(load_schedule(&conn), valid);
        for max_backups in [0, MAX_BACKUPS_LIMIT + 1] {
            assert!(Schedule {
                max_backups,
                ..valid.clone()
            }
            .validate()
            .is_err());
        }
        prefs::store(&conn, prefs::BACKUP_LOCAL_SCHEDULE_KEY, "garbage").unwrap();
        assert_eq!(load_schedule(&conn), Schedule::default());
    }

    #[test]
    fn state_reports_due_time_from_last_run() {
        let conn = Connection::open_in_memory().unwrap();
        prefs::initialize_preferences_table(&conn).unwrap();
        let dir = temp();
        assert_eq!(build_state(&conn, dir.path()).unwrap().next_due_at, None);
        let schedule = Schedule {
            enabled: true,
            ..Schedule::default()
        };
        prefs::store(
            &conn,
            prefs::BACKUP_LOCAL_SCHEDULE_KEY,
            &serde_json::to_string(&schedule).unwrap(),
        )
        .unwrap();
        prefs::store(&conn, prefs::BACKUP_LOCAL_LAST_RUN_KEY, "1000").unwrap();
        let state = build_state(&conn, dir.path()).unwrap();
        assert_eq!(state.last_run_at, Some(1000));
        assert_eq!(state.next_due_at, Some(1000 + Interval::Daily.seconds()));
    }
}
