//! "Forgot the password": erases the data of a profile so it opens again, empty and without a lock.
//! The lock encrypts nothing, so removing only the lock would let anyone past it; erasing the data is
//! what makes forgetting the password safe for the data that was there.
//!
//! The profile is open in the process that asks, so a helper process does the work after that one has
//! exited: it waits for the profile lock, erases, and starts the profile again.
use crate::error_log::{self, Level};
use crate::profile::{launcher, store, DEFAULT_ID, FLAG};
use crate::recovery::profile_lock::ProfileLock;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const ERASE_FLAG: &str = "--erase-profile";

/// Never erased from the folders of the Default profile: the other profiles live there, and so does
/// the choice made in the profile picker. `profile.lock` stays because the lock file is held while erasing.
const KEEP: &[&str] = &[
    "profiles",
    launcher::LAUNCHER_DIR,
    "launcher.json",
    "profile.lock",
];
const KEEP_LOCK_ONLY: &[&str] = &["profile.lock"];

fn parse(args: &[String]) -> Result<Option<String>, String> {
    let Some(position) = args.iter().position(|arg| arg == ERASE_FLAG) else {
        return Ok(None);
    };
    match args.get(position + 1).map(String::as_str) {
        Some(DEFAULT_ID) => Ok(Some(DEFAULT_ID.into())),
        Some(id) if store::valid_id(id) => Ok(Some(id.to_string())),
        _ => Err("profile.invalid_id".into()),
    }
}

/// The profile asked to be erased by `--erase-profile <id>` (`default` or a profile id), if any.
pub fn requested() -> Result<Option<String>, String> {
    parse(&std::env::args().collect::<Vec<_>>())
}

/// Removes everything inside `dir` except the names in `keep`. A folder that does not exist is fine.
/// An entry that cannot be removed (the web engine of the process that just closed may still hold it)
/// does not stop the others: the first failure is reported at the end, so a retry only has what is left.
pub fn erase_contents(dir: &Path, keep: &[&str]) -> Result<(), String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    let mut first_failure = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if keep.iter().any(|kept| name == *kept) {
            continue;
        }
        let path = entry.path();
        let removed = if path.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        if let Err(error) = removed {
            first_failure.get_or_insert(format!("{}: {error}", path.display()));
        }
    }
    first_failure.map_or(Ok(()), Err)
}

/// Where a profile keeps its data, and what stays in each place.
fn targets(
    local_root: &Path,
    default_data: &Path,
    id: &str,
) -> Vec<(PathBuf, &'static [&'static str])> {
    if id == DEFAULT_ID {
        let mut places = vec![(default_data.to_path_buf(), KEEP)];
        if local_root != default_data {
            places.push((local_root.to_path_buf(), KEEP));
        }
        places
    } else {
        let folder = store::folder(local_root, id);
        ["config", "data", "profile", "cache", "logs"]
            .iter()
            .map(|name| (folder.join(name), KEEP_LOCK_ONLY))
            .collect()
    }
}

/// Erases the data of a profile (the Default profile, or another one, whose name stays).
pub fn erase_profile(local_root: &Path, default_data: &Path, id: &str) -> Result<(), String> {
    let mut first_failure = None;
    for (dir, keep) in targets(local_root, default_data, id) {
        if let Err(error) = erase_contents(&dir, keep) {
            first_failure.get_or_insert(error);
        }
    }
    first_failure.map_or(Ok(()), Err)
}

/// Runs `attempt` until it succeeds or the time is up, pausing between tries.
fn retry(
    mut attempt: impl FnMut() -> Result<(), String>,
    timeout: Duration,
    pause: Duration,
) -> Result<(), String> {
    let started = Instant::now();
    loop {
        match attempt() {
            Ok(()) => return Ok(()),
            Err(error) if started.elapsed() >= timeout => return Err(error),
            Err(_) => std::thread::sleep(pause),
        }
    }
}

/// Waits until the profile is not open anywhere, so its files are not in use.
fn wait_until_free(data_dir: &Path, timeout: Duration) -> Result<ProfileLock, String> {
    let started = Instant::now();
    loop {
        match ProfileLock::acquire(data_dir) {
            Ok(lock) => return Ok(lock),
            Err(std::fs::TryLockError::WouldBlock) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

/// The helper process: waits for the profile to close, erases it and starts it again.
pub fn run_helper(local_root: &Path, default_data: &Path, id: &str) -> Result<(), String> {
    let data_dir = if id == DEFAULT_ID {
        default_data.to_path_buf()
    } else {
        store::folder(local_root, id).join("data")
    };
    let lock = wait_until_free(&data_dir, Duration::from_secs(60)).inspect_err(|details| {
        error_log::report(
            Level::Error,
            "app-lock",
            "the profile was not erased: it stayed open",
            details,
        );
    })?;
    // The web engine of the process that just closed can hold its files for a few seconds.
    let erased = retry(
        || erase_profile(local_root, default_data, id),
        Duration::from_secs(60),
        Duration::from_millis(500),
    );
    if let Err(details) = &erased {
        // The profile still starts (and is still locked if its database could not be removed), so the
        // person is not left with nothing; the error log says what stayed.
        error_log::report(
            Level::Error,
            "app-lock",
            "the profile could not be erased completely",
            &format!("{id}: {details}"),
        );
    }
    drop(lock);
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    std::process::Command::new(exe)
        .arg(FLAG)
        .arg(id)
        .spawn()
        .map(drop)
        .map_err(|error| error.to_string())
}

/// Starts the helper that erases a profile once the process that asked has exited.
pub fn start_helper(id: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    std::process::Command::new(exe)
        .arg(ERASE_FLAG)
        .arg(id)
        .spawn()
        .map(drop)
        .map_err(|error| {
            error_log::report(
                Level::Error,
                "app-lock",
                "the helper that erases the profile could not be started",
                &error.to_string(),
            );
            "lock.erase_failed".to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn erasing_removes_files_and_folders_but_keeps_the_named_entries() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("spark_browser.db"), "history");
        write(&dir.path().join("logs/spark-errors.log"), "log");
        write(&dir.path().join("profiles/abc/profile.json"), "{}");
        write(&dir.path().join("launcher.json"), "{}");
        erase_contents(dir.path(), KEEP).unwrap();
        assert!(!dir.path().join("spark_browser.db").exists());
        assert!(!dir.path().join("logs").exists());
        assert!(
            dir.path().join("profiles/abc/profile.json").is_file(),
            "other profiles are never touched"
        );
        assert!(dir.path().join("launcher.json").is_file());
        erase_contents(&dir.path().join("missing"), KEEP).unwrap();
    }

    #[test]
    fn erasing_the_default_profile_clears_both_of_its_folders_and_spares_the_others() {
        let data = tempfile::tempdir().unwrap();
        let local = tempfile::tempdir().unwrap();
        write(&data.path().join("spark_browser.db"), "history");
        write(&local.path().join("EBWebView/Default/Cookies"), "cookies");
        write(
            &local.path().join("profiles/0123456789ab/profile.json"),
            "{}",
        );
        write(
            &local
                .path()
                .join("profiles/0123456789ab/data/spark_browser.db"),
            "other",
        );
        write(&local.path().join("launcher/data/x"), "x");
        erase_profile(local.path(), data.path(), DEFAULT_ID).unwrap();
        assert!(!data.path().join("spark_browser.db").exists());
        assert!(!local.path().join("EBWebView").exists());
        assert!(local
            .path()
            .join("profiles/0123456789ab/data/spark_browser.db")
            .is_file());
        assert!(local
            .path()
            .join("profiles/0123456789ab/profile.json")
            .is_file());
        assert!(local.path().join("launcher/data/x").is_file());
    }

    #[test]
    fn where_both_folders_are_the_same_it_is_cleared_once() {
        let both = tempfile::tempdir().unwrap();
        write(&both.path().join("spark_browser.db"), "history");
        write(&both.path().join("profiles/abc/profile.json"), "{}");
        erase_profile(both.path(), both.path(), DEFAULT_ID).unwrap();
        assert!(!both.path().join("spark_browser.db").exists());
        assert!(both.path().join("profiles/abc/profile.json").is_file());
    }

    #[test]
    fn erasing_another_profile_keeps_its_name_and_nothing_else_of_it() {
        let root = tempfile::tempdir().unwrap();
        let work = store::create(root.path(), "Work", 1).unwrap();
        let folder = store::folder(root.path(), &work.id);
        write(&folder.join("data/spark_browser.db"), "history");
        write(&folder.join("profile/EBWebView/Cookies"), "cookies");
        let other = store::create(root.path(), "Home", 2).unwrap();
        let other_db = store::folder(root.path(), &other.id).join("data/spark_browser.db");
        write(&other_db, "mine");
        let data = tempfile::tempdir().unwrap();
        write(&data.path().join("spark_browser.db"), "default");
        erase_profile(root.path(), data.path(), &work.id).unwrap();
        assert!(!folder.join("data/spark_browser.db").exists());
        assert!(!folder.join("profile/EBWebView").exists());
        assert!(
            folder.join("profile.json").is_file(),
            "the profile keeps its name"
        );
        assert_eq!(store::get(root.path(), &work.id).unwrap().name, "Work");
        assert!(other_db.is_file());
        assert!(
            data.path().join("spark_browser.db").is_file(),
            "the Default profile is untouched"
        );
    }

    #[test]
    fn a_retry_keeps_trying_until_it_works_and_gives_up_with_the_last_error() {
        let mut tries = 0;
        let result = retry(
            || {
                tries += 1;
                if tries < 4 {
                    Err(format!("busy {tries}"))
                } else {
                    Ok(())
                }
            },
            Duration::from_secs(5),
            Duration::from_millis(5),
        );
        assert_eq!(result, Ok(()));
        assert_eq!(tries, 4);
        let mut count = 0;
        let failed = retry(
            || {
                count += 1;
                Err(format!("still busy {count}"))
            },
            Duration::from_millis(30),
            Duration::from_millis(10),
        );
        assert!(failed.unwrap_err().starts_with("still busy"));
        assert!(count >= 2, "it tried more than once");
    }

    #[cfg(windows)]
    #[test]
    fn an_entry_in_use_does_not_stop_the_others_and_a_retry_finishes_the_job() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("a_in_use.bin"), "locked");
        write(&dir.path().join("b_free.db"), "history");
        write(&dir.path().join("c_free/inner.txt"), "x");
        // Open without sharing, as a running web engine would.
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(dir.path().join("a_in_use.bin"))
            .unwrap();
        assert!(erase_contents(dir.path(), &[]).is_err());
        assert!(
            !dir.path().join("b_free.db").exists(),
            "the others were still removed"
        );
        assert!(!dir.path().join("c_free").exists());
        assert!(dir.path().join("a_in_use.bin").exists());
        drop(held);
        assert!(erase_contents(dir.path(), &[]).is_ok());
        assert!(!dir.path().join("a_in_use.bin").exists());
    }

    #[test]
    fn the_helper_waits_for_the_profile_to_be_free() {
        let dir = tempfile::tempdir().unwrap();
        let held = ProfileLock::acquire(dir.path()).unwrap();
        let path = dir.path().to_path_buf();
        let releaser = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            drop(held);
        });
        assert!(wait_until_free(&path, Duration::from_secs(5)).is_ok());
        releaser.join().unwrap();
        let busy = ProfileLock::acquire(dir.path()).unwrap();
        assert!(wait_until_free(dir.path(), Duration::from_millis(300)).is_err());
        drop(busy);
    }

    #[test]
    fn the_erase_argument_is_validated() {
        let list = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(parse(&list(&["x"])), Ok(None));
        assert_eq!(
            parse(&list(&["x", ERASE_FLAG, "default"])),
            Ok(Some("default".into()))
        );
        assert_eq!(
            parse(&list(&["x", ERASE_FLAG, "0123456789ab"])),
            Ok(Some("0123456789ab".into()))
        );
        for bad in ["../evil", "", "ABCDEF", "short"] {
            assert!(parse(&list(&["x", ERASE_FLAG, bad])).is_err(), "{bad}");
        }
        assert!(parse(&list(&["x", ERASE_FLAG])).is_err());
    }
}
