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
const SUFFIX: &str = ".spark-backup.json";
/// Suffix of history files written before the rename; they are still listed and restorable.
const LEGACY_SUFFIX: &str = ".veloce-backup.json";
const SNIFF_BYTES: usize = 160;
pub const DEFAULT_MAX_BACKUPS: u32 = 10;
pub const MAX_BACKUPS_LIMIT: u32 = 100;
pub const MAX_AGE_DAYS_LIMIT: u32 = 3650;
/// "Before replace" copies are an undo net, so they follow their own limits instead of the
/// user's version count: the newest few, for a month.
const PRE_RESTORE_KEEP: usize = 5;
const PRE_RESTORE_MAX_AGE_SECS: i64 = 30 * 24 * 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Manual,
    Scheduled,
    Exit,
    /// Safety copy taken right before a replace-mode restore, so it can be undone.
    PreRestore,
}
impl Reason {
    fn as_str(self) -> &'static str {
        match self {
            Reason::Manual => "manual",
            Reason::Scheduled => "scheduled",
            Reason::Exit => "exit",
            Reason::PreRestore => "pre_restore",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        match value {
            "manual" => Some(Reason::Manual),
            "scheduled" => Some(Reason::Scheduled),
            "exit" => Some(Reason::Exit),
            "pre_restore" => Some(Reason::PreRestore),
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
    /// Backups older than this many days are deleted; `None` keeps them until the count limit.
    #[serde(default)]
    pub max_age_days: Option<u32>,
}
impl Default for Schedule {
    fn default() -> Self {
        Self {
            // On from the start: a backup you forgot to turn on is the one you miss. A choice already
            // saved, including "off", is kept; only profiles that never touched the schedule change.
            enabled: true,
            interval: Interval::Daily,
            on_exit: true,
            max_backups: DEFAULT_MAX_BACKUPS,
            max_age_days: None,
        }
    }
}
impl Schedule {
    fn validate(&self) -> Result<(), String> {
        if (1..=MAX_BACKUPS_LIMIT).contains(&self.max_backups)
            && self
                .max_age_days
                .is_none_or(|days| (1..=MAX_AGE_DAYS_LIMIT).contains(&days))
        {
            Ok(())
        } else {
            Err("backup.schedule_invalid".into())
        }
    }
    fn retention(&self) -> Retention {
        Retention {
            keep: self.max_backups,
            max_age_secs: self.max_age_days.map(|days| i64::from(days) * 24 * 3600),
        }
    }
}

/// What the history keeps. Applied after every backup, when the schedule changes and when the
/// state is first read, so an age limit also works while no backup is due.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Retention {
    keep: u32,
    max_age_secs: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    pub id: String,
    pub created_at: i64,
    pub reason: Reason,
    pub size_bytes: u64,
    pub encrypted: bool,
}

/// The most recent backup attempt that failed, kept until a later attempt succeeds. Only a stable
/// error code is stored, never the native message, which may contain file paths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub at: i64,
    pub reason: Reason,
    pub code: String,
}
/// `backup.some_code` or `backup.some_code: detail` becomes the code; anything else is generic.
fn failure_code(error: &str) -> String {
    let head = error.split(':').next().unwrap_or("").trim();
    let valid = head.split_once('.').is_some_and(|(area, name)| {
        !area.is_empty()
            && !name.is_empty()
            && head
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.')
    });
    if valid {
        head.to_owned()
    } else {
        "operation.failed".to_owned()
    }
}
fn load_failure(conn: &Connection) -> Option<Failure> {
    prefs::load(conn, prefs::BACKUP_LOCAL_FAILURE_KEY)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

#[derive(Clone, Debug, Serialize)]
pub struct LocalState {
    pub schedule: Schedule,
    pub encryption_enabled: bool,
    pub last_run_at: Option<i64>,
    /// Set while the latest attempt failed; cleared by the next success.
    pub last_failure: Option<Failure>,
    /// Epoch seconds at which the next scheduled backup is due; `None` when disabled.
    pub next_due_at: Option<i64>,
    pub directory: String,
    pub entries: Vec<Entry>,
    pub total_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOutcome {
    pub created: bool,
    /// File name of the snapshot written by this call, when one was.
    pub snapshot_id: Option<String>,
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
    let rest = name
        .strip_prefix(PREFIX)?
        .strip_suffix(SUFFIX)
        .or_else(|| name.strip_prefix(PREFIX)?.strip_suffix(LEGACY_SUFFIX))?;
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
        || String::from_utf8_lossy(&head).contains(crypto::LEGACY_ENCRYPTED_FORMAT)
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

/// Ids to delete. Regular snapshots follow the count and age limits but the newest one always
/// stays, so retention can never leave the history empty. Safety copies follow their own limits.
fn expired(entries: &[Entry], retention: Retention, now: i64) -> Vec<String> {
    let too_old =
        |entry: &Entry, limit: Option<i64>| limit.is_some_and(|l| now - entry.created_at > l);
    let (safety, regular): (Vec<_>, Vec<_>) = entries
        .iter()
        .partition(|entry| entry.reason == Reason::PreRestore);
    let regular_ids = regular.iter().enumerate().filter(|(index, entry)| {
        *index >= retention.keep as usize || (*index > 0 && too_old(entry, retention.max_age_secs))
    });
    let safety_ids = safety.iter().enumerate().filter(|(index, entry)| {
        *index >= PRE_RESTORE_KEEP || too_old(entry, Some(PRE_RESTORE_MAX_AGE_SECS))
    });
    regular_ids
        .chain(safety_ids)
        .map(|(_, entry)| entry.id.clone())
        .collect()
}

/// Applies the retention policy and returns how many snapshots were removed.
fn prune(dir: &Path, retention: Retention, now: i64) -> Result<usize, String> {
    let mut removed = 0;
    for id in expired(&list(dir)?, retention, now) {
        if fs::remove_file(dir.join(&id)).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

/// Writes atomically under a name that never replaces an existing snapshot.
fn write_snapshot(
    dir: &Path,
    created_at: i64,
    reason: Reason,
    bytes: &[u8],
) -> Result<String, String> {
    for offset in 0..60 {
        let name = file_name(created_at + offset, reason);
        let path = dir.join(&name);
        if !path.exists() {
            files::write_bytes(&path, bytes)?;
            return Ok(name);
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
        last_failure: load_failure(conn),
        next_due_at,
        directory: dir.to_string_lossy().into_owned(),
        total_bytes: entries.iter().map(|entry| entry.size_bytes).sum(),
        entries,
    })
}

pub async fn state(app: &AppHandle) -> Result<LocalState, String> {
    let dir = directory(app)?;
    crate::database::executor::run(app.clone(), move |conn| {
        // Reading the history is when an age limit takes effect for users with nothing due.
        prune(&dir, load_schedule(conn).retention(), now())?;
        build_state(conn, &dir)
    })
    .await
}

pub async fn set_schedule(app: &AppHandle, schedule: Schedule) -> Result<LocalState, String> {
    schedule.validate()?;
    let dir = directory(app)?;
    crate::database::executor::run(app.clone(), move |conn| {
        let raw = serde_json::to_string(&schedule).map_err(|e| e.to_string())?;
        prefs::store(conn, prefs::BACKUP_LOCAL_SCHEDULE_KEY, &raw)?;
        // A lowered limit applies immediately instead of at the next backup.
        prune(&dir, schedule.retention(), now())?;
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

/// Captures and stores a snapshot, remembering whether the attempt worked so the status panel
/// can show a failure after a restart. With `only_if_changed`, an unchanged document only
/// refreshes the schedule clock instead of piling up identical files.
pub async fn create(
    app: &AppHandle,
    preferences: BackupPreferences,
    reason: Reason,
    only_if_changed: bool,
) -> Result<CreateOutcome, String> {
    let result = create_snapshot(app, preferences, reason, only_if_changed).await;
    let failure = result.as_ref().err().map(|error| Failure {
        at: now(),
        reason,
        code: failure_code(error),
    });
    // Bookkeeping must never replace the real outcome of the backup.
    let _ = crate::database::executor::run(app.clone(), move |conn| match &failure {
        Some(failure) => serde_json::to_string(failure)
            .map_err(|e| e.to_string())
            .and_then(|raw| prefs::store(conn, prefs::BACKUP_LOCAL_FAILURE_KEY, &raw)),
        None => prefs::remove(conn, prefs::BACKUP_LOCAL_FAILURE_KEY),
    })
    .await;
    result
}

async fn create_snapshot(
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
    let (skip, password, retention) = {
        let (dir, fingerprint) = (dir.clone(), fingerprint.clone());
        crate::database::executor::run(app.clone(), move |conn| {
            let schedule = load_schedule(conn);
            let unchanged = only_if_changed
                && prefs::load(conn, prefs::BACKUP_LOCAL_FINGERPRINT_KEY)?.as_deref()
                    == Some(fingerprint.as_str())
                && !list(&dir)?.is_empty();
            if unchanged {
                prefs::store(conn, prefs::BACKUP_LOCAL_LAST_RUN_KEY, &now().to_string())?;
                return Ok((true, None, schedule.retention()));
            }
            let credentials = &worker.state::<service::BackupService>().sync.credentials;
            let password = encryption::write_password(conn, credentials)?;
            Ok((false, password, schedule.retention()))
        })
        .await?
    };

    let mut snapshot_id = None;
    if !skip {
        let dir = dir.clone();
        snapshot_id = Some(
            tauri::async_runtime::spawn_blocking(move || {
                // Key derivation and fsync stay off the database lock.
                let bytes = files::encode(&document, password.as_deref())?;
                let id = write_snapshot(&dir, now(), reason, &bytes)?;
                prune(&dir, retention, now())?;
                Ok::<_, String>(id)
            })
            .await
            .map_err(|e| e.to_string())??,
        );
    } else {
        // Nothing new to store, but an age limit may still have expired older entries.
        let dir = dir.clone();
        tauri::async_runtime::spawn_blocking(move || prune(&dir, retention, now()))
            .await
            .map_err(|e| e.to_string())??;
    }
    // Safety copies must not postpone the schedule or mask a later unchanged check.
    let record_run = reason != Reason::PreRestore;

    let dir_for_state = dir;
    crate::database::executor::run(app.clone(), move |conn| {
        if !skip && record_run {
            prefs::store(conn, prefs::BACKUP_LOCAL_FINGERPRINT_KEY, &fingerprint)?;
            prefs::store(conn, prefs::BACKUP_LOCAL_LAST_RUN_KEY, &now().to_string())?;
        }
        Ok(CreateOutcome {
            created: !skip,
            snapshot_id,
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

/// Test hooks: the helpers stay private, the legacy tests only need their behavior.
#[cfg(test)]
pub(super) fn file_name_for_test(created_at: i64, reason: &str) -> String {
    file_name(created_at, Reason::parse(reason).unwrap())
}
#[cfg(test)]
pub(super) fn parses_for_test(name: &str) -> bool {
    parse_name(name).is_some()
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
            "../backup-1-manual.spark-backup.json",
            "backup-1-manual.spark-backup.json.bak",
            "backup--manual.spark-backup.json",
            "backup-1-other.spark-backup.json",
            "backup-1.spark-backup.json",
            "notes.txt",
            "backup-1234567890123-manual.spark-backup.json",
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
    fn pre_restore_snapshots_roundtrip_and_are_listed_with_their_reason() {
        let name = file_name(1_700_000_000, Reason::PreRestore);
        assert_eq!(parse_name(&name), Some((1_700_000_000, Reason::PreRestore)));
        let dir = temp();
        let written = write_snapshot(dir.path(), 5, Reason::PreRestore, b"{}").unwrap();
        let entries = list(dir.path()).unwrap();
        assert_eq!(entries[0].id, written);
        assert_eq!(entries[0].reason, Reason::PreRestore);
    }

    #[test]
    fn list_orders_newest_first_flags_encryption_and_ignores_foreign_files() {
        let dir = temp();
        write_snapshot(
            dir.path(),
            100,
            Reason::Manual,
            b"{\n  \"format\": \"spark-backup\"\n}",
        )
        .unwrap();
        write_snapshot(
            dir.path(),
            300,
            Reason::Exit,
            b"{\n  \"format\": \"spark-backup-encrypted\"\n}",
        )
        .unwrap();
        write_snapshot(dir.path(), 200, Reason::Scheduled, b"{}").unwrap();
        fs::write(dir.path().join("readme.txt"), "x").unwrap();
        fs::write(dir.path().join(".spark-backup-abc.tmp"), "x").unwrap();
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
        assert_eq!(prune(dir.path(), keep(2), NOW).unwrap(), 3);
        assert_eq!(
            list(dir.path())
                .unwrap()
                .iter()
                .map(|e| e.created_at)
                .collect::<Vec<_>>(),
            [50, 40]
        );
        assert_eq!(prune(dir.path(), keep(2), NOW).unwrap(), 0);
    }

    const NOW: i64 = 10_000_000;
    const DAY: i64 = 24 * 3600;
    fn keep(count: u32) -> Retention {
        Retention {
            keep: count,
            max_age_secs: None,
        }
    }
    fn entry(age_days: i64, reason: Reason) -> Entry {
        let created_at = NOW - age_days * DAY;
        Entry {
            id: file_name(created_at, reason),
            created_at,
            reason,
            size_bytes: 1,
            encrypted: false,
        }
    }
    fn expired_ages(entries: &[Entry], retention: Retention) -> Vec<i64> {
        let ids = expired(entries, retention, NOW);
        entries
            .iter()
            .filter(|e| ids.contains(&e.id))
            .map(|e| (NOW - e.created_at) / DAY)
            .collect()
    }

    #[test]
    fn failures_keep_only_a_stable_code_and_roundtrip_through_the_state() {
        assert_eq!(
            failure_code("backup.password_missing"),
            "backup.password_missing"
        );
        assert_eq!(
            failure_code("backup.invalid_document: expected value at line 1"),
            "backup.invalid_document"
        );
        for leaky in [
            "Access is denied. (os error 5) C:\\Users\\me\\backups",
            "No such file: /home/me/x",
            "",
            ":",
            "Backup.Upper",
        ] {
            assert_eq!(failure_code(leaky), "operation.failed", "{leaky}");
        }
        let conn = Connection::open_in_memory().unwrap();
        prefs::initialize_preferences_table(&conn).unwrap();
        let dir = temp();
        assert_eq!(build_state(&conn, dir.path()).unwrap().last_failure, None);
        let failure = Failure {
            at: 42,
            reason: Reason::Scheduled,
            code: "backup.crypto_failed".into(),
        };
        prefs::store(
            &conn,
            prefs::BACKUP_LOCAL_FAILURE_KEY,
            &serde_json::to_string(&failure).unwrap(),
        )
        .unwrap();
        assert_eq!(
            build_state(&conn, dir.path()).unwrap().last_failure,
            Some(failure)
        );
        prefs::store(&conn, prefs::BACKUP_LOCAL_FAILURE_KEY, "garbage").unwrap();
        assert_eq!(build_state(&conn, dir.path()).unwrap().last_failure, None);
    }

    #[test]
    fn age_limit_removes_old_snapshots_but_always_keeps_the_newest() {
        let aged = Retention {
            keep: 50,
            max_age_secs: Some(10 * DAY),
        };
        let entries: Vec<_> = [1, 5, 11, 40]
            .iter()
            .map(|d| entry(*d, Reason::Scheduled))
            .collect();
        assert_eq!(expired_ages(&entries, aged), [11, 40]);
        // Everything is older than the limit: the newest stays as the last copy.
        let stale: Vec<_> = [20, 30]
            .iter()
            .map(|d| entry(*d, Reason::Scheduled))
            .collect();
        assert_eq!(expired_ages(&stale, aged), [30]);
    }

    #[test]
    fn count_and_age_limits_combine() {
        let both = Retention {
            keep: 2,
            max_age_secs: Some(10 * DAY),
        };
        let entries: Vec<_> = [1, 2, 3, 20]
            .iter()
            .map(|d| entry(*d, Reason::Manual))
            .collect();
        assert_eq!(expired_ages(&entries, both), [3, 20]);
    }

    #[test]
    fn safety_copies_have_their_own_limits_and_do_not_use_the_user_count() {
        let tight = Retention {
            keep: 1,
            max_age_secs: Some(DAY),
        };
        let mut entries = vec![entry(0, Reason::Scheduled)];
        // Seven safety copies, the oldest beyond 30 days: age and the cap of five both apply.
        entries.extend(
            [0, 1, 2, 3, 4, 5, 40]
                .iter()
                .map(|d| entry(*d, Reason::PreRestore)),
        );
        // The five newest safety copies survive a one-version, one-day policy.
        let removed = expired_ages(&entries, tight);
        assert_eq!(removed, [5, 40]);
        assert!(!expired(&entries, tight, NOW).contains(&entries[0].id));
    }

    #[test]
    fn retention_never_empties_the_history_or_touches_foreign_files() {
        let dir = temp();
        write_snapshot(dir.path(), NOW - 99 * DAY, Reason::Scheduled, b"{}").unwrap();
        fs::write(dir.path().join("keep-me.txt"), "x").unwrap();
        let aged = Retention {
            keep: 1,
            max_age_secs: Some(DAY),
        };
        assert_eq!(prune(dir.path(), aged, NOW).unwrap(), 0);
        assert_eq!(list(dir.path()).unwrap().len(), 1);
        assert!(dir.path().join("keep-me.txt").exists());
    }

    #[test]
    fn schedules_saved_before_the_age_limit_existed_still_load() {
        let old = r#"{"enabled":true,"interval":"daily","on_exit":false,"max_backups":7}"#;
        let schedule: Schedule = serde_json::from_str(old).unwrap();
        assert_eq!(schedule.max_age_days, None);
        assert!(schedule.validate().is_ok());
        for days in [0, MAX_AGE_DAYS_LIMIT + 1] {
            let bad = Schedule {
                max_age_days: Some(days),
                ..schedule.clone()
            };
            assert!(bad.validate().is_err());
        }
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
            max_age_days: Some(90),
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
        // Untouched profiles are on by default, so a first backup is due right away.
        let untouched = build_state(&conn, dir.path()).unwrap();
        assert!(untouched.schedule.enabled && untouched.schedule.on_exit);
        assert!(untouched.next_due_at.is_some());
        // An explicit "off" is respected and has nothing due.
        let off = Schedule {
            enabled: false,
            on_exit: false,
            ..Schedule::default()
        };
        prefs::store(
            &conn,
            prefs::BACKUP_LOCAL_SCHEDULE_KEY,
            &serde_json::to_string(&off).unwrap(),
        )
        .unwrap();
        let declined = build_state(&conn, dir.path()).unwrap();
        assert!(!declined.schedule.enabled && !declined.schedule.on_exit);
        assert_eq!(declined.next_due_at, None);
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
