//! Private windows: each one is a separate process with a disposable profile,
//! so nothing it stores reaches the normal profile.
//!
//! The main process creates `<local data>/private-sessions/<session>` and starts
//! this executable again with `--private-root` and `--private-source`; it does
//! not wait for that process. The child redirects every app directory into the
//! session folder, copies favorites and settings from the normal database, and
//! removes its folder after it closes. The main process removes folders left
//! behind by crashes at startup.

use crate::database::connection::{initialize_schema, resolve_db_path};
use crate::database::preferences_repo::initialize_preferences_table;
use crate::recovery::profile_lock::ProfileLock;
use rusqlite::Connection;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};
use tauri::{AppHandle, Manager};

pub const SESSIONS_DIR: &str = "private-sessions";
const ROOT_FLAG: &str = "--private-root";
const SOURCE_FLAG: &str = "--private-source";
const CLEANUP_FLAG: &str = "--private-cleanup";
const DATABASE_FILE: &str = "spark_browser.db";
/// Tables copied from the normal profile when a private window opens.
const COPIED_TABLES: &[&str] = &[
    "bookmark_folders",
    "bookmarks",
    "site_zoom",
    "site_search_shortcuts",
    "new_tab_settings",
    "app_preferences",
];
/// Backup sync secrets stay in the normal profile and are never copied.
const EXCLUDED_PREFERENCE_KEYS: &[&str] = &["backup_sync_secret_v1", "backup_sync_credential_v2"];
const EXCLUDED_PREFERENCE_PREFIX: &str = "backup_sync";

#[derive(Clone, Debug)]
pub struct Launch {
    root: PathBuf,
    source: PathBuf,
}

pub enum Mode {
    Normal,
    Private(Launch),
    Cleanup(PathBuf),
}

/// Managed state describing whether this process is a private window.
#[derive(Default)]
pub struct PrivateRole {
    launch: Option<Launch>,
}

impl PrivateRole {
    pub fn from_mode(mode: Mode) -> Self {
        match mode {
            Mode::Private(launch) => Self {
                launch: Some(launch),
            },
            Mode::Normal | Mode::Cleanup(_) => Self::default(),
        }
    }
    pub fn launch(&self) -> Option<&Launch> {
        self.launch.as_ref()
    }
    pub fn is_private(&self) -> bool {
        self.launch.is_some()
    }
}

impl Mode {
    pub fn from_args() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().collect();
        if let Some(root) = flag_value(&args, CLEANUP_FLAG) {
            return Ok(Self::Cleanup(checked_root(&root)?));
        }
        match (flag_value(&args, ROOT_FLAG), flag_value(&args, SOURCE_FLAG)) {
            (None, None) => Ok(Self::Normal),
            (Some(root), Some(source)) => Ok(Self::Private(Launch {
                root: checked_root(&root)?,
                source: PathBuf::from(source),
            })),
            _ => Err("private.invalid_arguments".into()),
        }
    }
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

/// Accepts only an absolute folder named like a session, directly inside `private-sessions`.
fn checked_root(raw: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(raw);
    let under_sessions = root.parent().and_then(Path::file_name) == Some(OsStr::new(SESSIONS_DIR));
    let plain_name = root
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| {
            !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        });
    if root.is_absolute() && under_sessions && plain_name {
        Ok(root)
    } else {
        Err("private.invalid_root".into())
    }
}

/// Points every app directory of a private window into its session folder.
pub fn configure(context: &mut tauri::Context<tauri::Wry>, launch: Option<&Launch>) {
    let Some(launch) = launch else {
        return;
    };
    let directory = &launch.root;
    context.config_mut().app.app_directories_override =
        Some(AppDirectoriesOverride::Directories(AppDirectoryOverrides {
            config: Some(directory.join("config")),
            data: Some(directory.join("data")),
            local_data: Some(directory.join("profile")),
            cache: Some(directory.join("cache")),
            log: Some(directory.join("logs")),
        }));
}

/// Opens a new private window as a separate process.
pub fn open_window(app: &AppHandle, role: &PrivateRole) -> Result<(), String> {
    if role.is_private() {
        return Err("private.already_private".into());
    }
    let local = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("private.paths: {e}"))?;
    let source = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("private.paths: {e}"))?;
    let root = local.join(SESSIONS_DIR).join(session_name());
    std::fs::create_dir_all(&root).map_err(|e| format!("private.create_failed: {e}"))?;
    let exe = std::env::current_exe().map_err(|e| format!("private.spawn_failed: {e}"))?;
    if let Err(error) = Command::new(exe)
        .arg(ROOT_FLAG)
        .arg(&root)
        .arg(SOURCE_FLAG)
        .arg(&source)
        .spawn()
    {
        crate::error_log::ignored(std::fs::remove_dir_all(&root));
        return Err(format!("private.spawn_failed: {error}"));
    }
    Ok(())
}

fn session_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    format!("{:x}-{:x}", std::process::id(), nanos)
}

/// Fills a fresh session database with the favorites and settings of the normal profile.
pub fn seed_profile(app: &AppHandle) -> Result<(), String> {
    let role = app.state::<PrivateRole>();
    let Some(launch) = role.launch() else {
        return Ok(());
    };
    let source_db = launch.source.join(DATABASE_FILE);
    if !source_db.is_file() {
        return Ok(());
    }
    let target = resolve_db_path(app)?;
    let conn = Connection::open(&target).map_err(|e| format!("private.open_failed: {e}"))?;
    initialize_schema(&conn)?;
    initialize_preferences_table(&conn).map_err(|e| e.to_string())?;
    copy_profile(&conn, &source_db)
}

pub fn copy_profile(conn: &Connection, source_db: &Path) -> Result<(), String> {
    let source = source_db.to_str().ok_or("private.invalid_source")?;
    conn.execute("ATTACH DATABASE ?1 AS source", [source])
        .map_err(|e| format!("private.attach_failed: {e}"))?;
    let copied = COPIED_TABLES
        .iter()
        .try_for_each(|table| copy_table(conn, table));
    let detached = conn
        .execute_batch("DETACH DATABASE source")
        .map_err(|e| format!("private.detach_failed: {e}"));
    copied.and(detached)?;
    let mut settings = crate::database::new_tab_repo::load(conn)?;
    settings.spotify_enabled = false;
    settings.spotify_client_id.clear();
    settings.audio_background = Default::default();
    crate::database::new_tab_repo::save(conn, &settings)
}

fn copy_table(conn: &Connection, table: &str) -> Result<(), String> {
    let target = table_columns(conn, "main", table)?;
    let source = table_columns(conn, "source", table)?;
    let shared: Vec<&str> = target
        .iter()
        .map(String::as_str)
        .filter(|column| source.iter().any(|name| name == *column))
        .collect();
    if shared.is_empty() {
        return Ok(());
    }
    let columns = shared
        .iter()
        .map(|column| format!("\"{column}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let filter = if table == "app_preferences" {
        preference_filter()
    } else {
        String::new()
    };
    conn.execute_batch(&format!(
        "DELETE FROM main.\"{table}\";
         INSERT INTO main.\"{table}\" ({columns}) SELECT {columns} FROM source.\"{table}\"{filter};"
    ))
    .map_err(|e| format!("private.copy_failed: {e}"))
}

fn preference_filter() -> String {
    let keys = EXCLUDED_PREFERENCE_KEYS
        .iter()
        .map(|key| format!("'{key}'"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        " WHERE key NOT IN ({keys}) AND substr(key, 1, {}) <> '{EXCLUDED_PREFERENCE_PREFIX}'",
        EXCLUDED_PREFERENCE_PREFIX.len()
    )
}

fn table_columns(conn: &Connection, schema: &str, table: &str) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare(&format!("PRAGMA {schema}.table_info(\"{table}\")"))
        .map_err(|e| format!("private.schema_failed: {e}"))?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("private.schema_failed: {e}"))?;
    names
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("private.schema_failed: {e}"))
}

/// Main process only: removes session folders whose profile lock is free,
/// which means their window is gone (for example after a crash).
pub fn purge_stale_sessions(app: &AppHandle) -> Result<(), String> {
    let local = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("private.paths: {e}"))?;
    purge_stale_in(&local.join(SESSIONS_DIR))
}

pub fn purge_stale_in(sessions: &Path) -> Result<(), String> {
    let entries = match std::fs::read_dir(sessions) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("private.scan_failed: {error}")),
    };
    for entry in entries.flatten() {
        let root = entry.path();
        // A running private window holds its profile lock, so it is kept.
        let idle = ProfileLock::acquire(&root.join("data")).is_ok();
        if idle {
            crate::error_log::ignored(std::fs::remove_dir_all(&root));
        }
    }
    Ok(())
}

/// Starts a helper that removes this window's session folder once this process has exited.
/// WebView2 keeps its files locked until then, so the helper retries for about a minute.
pub fn schedule_cleanup(app: &AppHandle) {
    let Some(root) = app
        .try_state::<PrivateRole>()
        .and_then(|role| role.launch().map(|launch| launch.root.clone()))
    else {
        return;
    };
    if let Ok(exe) = std::env::current_exe() {
        crate::error_log::ignored(Command::new(exe).arg(CLEANUP_FLAG).arg(&root).spawn());
    }
}

/// Runs in the cleanup helper process, which never opens a window.
pub fn remove_session(root: &Path) -> Result<(), String> {
    for _ in 0..120 {
        if !root.exists() {
            return Ok(());
        }
        if std::fs::remove_dir_all(root).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err("private.cleanup_failed".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_session_folders_are_accepted_as_roots() {
        let base = std::env::temp_dir().join(SESSIONS_DIR);
        assert!(checked_root(&base.join("1a-2b").to_string_lossy()).is_ok());
        assert!(checked_root(&std::env::temp_dir().join("1a-2b").to_string_lossy()).is_err());
        assert!(checked_root(&base.join("..").join("x").to_string_lossy()).is_err());
        assert!(checked_root(&base.join("a b").to_string_lossy()).is_err());
    }

    #[test]
    fn stale_scan_keeps_running_sessions_and_removes_idle_ones() {
        let sessions = tempfile::tempdir().unwrap();
        let running = sessions.path().join("running");
        let idle = sessions.path().join("idle");
        let _lock = ProfileLock::acquire(&running.join("data")).unwrap();
        std::fs::create_dir_all(idle.join("data")).unwrap();
        purge_stale_in(sessions.path()).unwrap();
        assert!(running.exists());
        assert!(!idle.exists());
    }

    #[test]
    fn profile_copy_keeps_settings_and_favorites_but_not_backup_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("source.db");
        let target_path = dir.path().join("target.db");
        let source = Connection::open(&source_path).unwrap();
        initialize_schema(&source).unwrap();
        initialize_preferences_table(&source).unwrap();
        crate::database::new_tab_repo::save(
            &source,
            &crate::database::new_tab_repo::Settings {
                spotify_enabled: true,
                spotify_client_id: "a".repeat(32),
                audio_background: crate::database::new_tab_repo::AudioBackground {
                    enabled: true,
                    color: "#112233".into(),
                    intensity: 1.5,
                    style: "radial".into(),
                },
                ..Default::default()
            },
        )
        .unwrap();
        source
            .execute(
                "INSERT INTO app_preferences (key, value) VALUES
                 ('theme', 'dark'),
                 ('backup_sync_secret_v1', 'secret'),
                 ('backup_sync_credential_v2', 'token')",
                [],
            )
            .unwrap();
        source
            .execute(
                "INSERT INTO bookmarks (title, url, icon, created_at)
                 VALUES ('Docs', 'https://example.com/docs', '', 1)",
                [],
            )
            .unwrap();
        drop(source);

        let target = Connection::open(&target_path).unwrap();
        initialize_schema(&target).unwrap();
        initialize_preferences_table(&target).unwrap();
        copy_profile(&target, &source_path).unwrap();
        let settings = crate::database::new_tab_repo::load(&target).unwrap();
        assert!(!settings.spotify_enabled);
        assert!(settings.spotify_client_id.is_empty());
        assert_eq!(settings.audio_background, Default::default());

        let keys: Vec<String> = target
            .prepare("SELECT key FROM app_preferences ORDER BY key")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(keys, vec!["theme".to_string()]);

        let favorites: i64 = target
            .query_row(
                "SELECT COUNT(*) FROM bookmarks WHERE url = 'https://example.com/docs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(favorites, 1);
    }
}
