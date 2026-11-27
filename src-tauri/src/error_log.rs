//! Local log and error notifications, with four severity levels.
//!
//! Everything that goes wrong is appended to `<data directory>/logs/spark-errors.log`, each entry
//! tagged with its [`Level`]:
//!
//! | Level | Meaning | Examples | Notification |
//! | --- | --- | --- | --- |
//! | `Info` | Notable events that are not problems | start, normal close, a freeze that ended | no |
//! | `Warning` | A failure Spark ignored and recovered from | a discarded `Result` (see [`ignored`]) | no |
//! | `Error` | Something failed and the user may notice | native errors, script errors | yes |
//! | `Critical` | The app itself is in trouble | a panic, a freeze, a run that did not end cleanly | yes |
//!
//! `Error` and `Critical` also reach the browser shell, which shows a card with buttons to view the
//! error and to send it as a GitHub issue. Nothing is sent by Spark itself. Entries are redacted
//! first: the user's home folder becomes `~` and page addresses lose their path and query.
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    fs::{self, OpenOptions},
    io::Write,
    panic::Location,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

const FILE_NAME: &str = "spark-errors.log";
/// The log is moved to `.old` once it passes this size, which bounds disk use to about twice it.
const MAX_BYTES: u64 = 512 * 1024;
/// The window is reported as not responding after this many seconds without answering.
const HANG_AFTER_SECONDS: u64 = 10;
/// The same entry is written to the log at most once in this many seconds (never for `Critical`).
const LOG_REPEAT_SECONDS: u64 = 10;
/// The same error is shown at most once in this many seconds, so a loop cannot flood the screen.
const REPEAT_SECONDS: u64 = 60;
/// Notifications per minute, across all errors.
const MAX_PER_MINUTE: usize = 5;
/// Distinct entries remembered for de-duplication before the table is emptied.
const MAX_REMEMBERED: usize = 500;
const MAX_SUMMARY: usize = 300;
const MAX_DETAILS: usize = 16_000;
const SESSION_MARKER: &str = "session.running";

static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();
static HANDLE: OnceLock<AppHandle> = OnceLock::new();
static HEARTBEAT: AtomicU64 = AtomicU64::new(0);
static COUNTER: AtomicU64 = AtomicU64::new(0);
/// Notifications shown in the last minute: (time, key).
static SHOWN: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());
/// Each entry's last write to the log and how many repeats were skipped since: key -> (time, skipped).
static WRITTEN: Mutex<Option<HashMap<String, (u64, u32)>>> = Mutex::new(None);
/// The most recent things the app was asked to do, oldest first. A freeze has no error to show, so
/// its log entry lists these: the last lines are what the app was doing when it stopped answering.
static TRAIL: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
const TRAIL_LENGTH: usize = 40;
/// Set when the previous run left its marker behind, which means it did not close normally.
static PREVIOUS_RUN_ENDED_BADLY: OnceLock<bool> = OnceLock::new();

/// How serious an entry is. See the module documentation for what each level means.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Info,
    Warning,
    Error,
    Critical,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Warning => "WARN",
            Level::Error => "ERROR",
            Level::Critical => "CRITICAL",
        }
    }

    /// Only the two serious levels interrupt the user with a card.
    fn notifies(self) -> bool {
        matches!(self, Level::Error | Level::Critical)
    }
}

/// What the shell shows in the error notification.
#[derive(Clone, Serialize)]
pub struct ErrorReport {
    pub id: String,
    pub level: Level,
    /// `panic`, `hang`, `native`, `interface`, `new-tab`, `shutdown` or `unclean-exit`.
    pub kind: String,
    pub summary: String,
    pub details: String,
    pub version: &'static str,
    pub os: &'static str,
    pub time: String,
}

/// Folder that holds the log, so Settings or documentation can point users to it.
pub fn directory(data_directory: &Path) -> PathBuf {
    data_directory.join("logs")
}

fn open_log(data_directory: &Path) -> bool {
    let directory = directory(data_directory);
    fs::create_dir_all(&directory).is_ok() && LOG_FILE.set(directory.join(FILE_NAME)).is_ok()
}

/// Records a launch rejected before acquiring the profile, without touching its run marker.
pub fn rejected_launch(data_directory: &Path, level: Level, summary: &str, details: &str) {
    if open_log(data_directory) {
        report(level, "startup", summary, details);
    }
}

/// Installs the panic hook. Call once, as early as the data directory is known. The previous hook
/// still runs, so the console output in development builds is unchanged.
pub fn install(data_directory: &Path) {
    let directory = directory(data_directory);
    if !open_log(data_directory) {
        return;
    }
    let marker = directory.join(SESSION_MARKER);
    let _ = PREVIOUS_RUN_ENDED_BADLY.set(marker.exists());
    let _ = fs::write(&marker, unix_seconds().to_string());
    report(
        Level::Info,
        "lifecycle",
        &format!("Spark {} started", env!("CARGO_PKG_VERSION")),
        "",
    );
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let location = info
            .location()
            .map_or_else(|| "unknown location".into(), ToString::to_string);
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|m| (*m).to_owned())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-text panic payload".into());
        let backtrace = std::backtrace::Backtrace::force_capture();
        // Reported from a new thread: the panicking one may hold a lock the notification needs.
        let summary = format!(
            "Internal error in thread '{}': {message}",
            thread.name().unwrap_or("unnamed")
        );
        let details = format!("at {location}\n{backtrace}");
        let _ = std::thread::Builder::new()
            .name("error-log-panic".into())
            .spawn(move || report(Level::Critical, "panic", &summary, &details))
            .map(|handle| handle.join());
        previous(info);
    }));
}

/// Watches the main thread: every second it asks the event loop to update a counter, and a counter
/// that stops moving for `HANG_AFTER_SECONDS` is a freeze. It is logged when it starts and
/// reported, with its duration, when the window answers again. That tells a freeze apart from a
/// crash and shows how long it lasted.
pub fn watch(app: &AppHandle) {
    if LOG_FILE.get().is_none() {
        return;
    }
    let _ = HANDLE.set(app.clone());
    // `watch` runs in the application setup, on the main thread: its stack is the one to look at in a freeze.
    crate::hang_stack::remember_main_thread();
    if PREVIOUS_RUN_ENDED_BADLY.get().copied().unwrap_or(false) {
        // Wait until the interface is listening before telling the user.
        let _ = std::thread::Builder::new()
            .name("error-log-previous-run".into())
            .spawn(|| {
                std::thread::sleep(Duration::from_secs(6));
                report(
                    Level::Critical,
                    "unclean-exit",
                    "Spark did not close normally the last time it ran",
                    "The previous run left its running marker behind, so it ended without a clean \
                     shutdown: a crash, a freeze that was force-closed, or the computer losing power.",
                );
            });
    }
    let app = app.clone();
    let _ = std::thread::Builder::new()
        .name("error-log-watchdog".into())
        .spawn(move || {
            let mut reported: Option<u64> = None;
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let now = unix_seconds();
                let beat = HEARTBEAT.load(Ordering::Relaxed);
                ignored(
                    app.run_on_main_thread(|| HEARTBEAT.store(unix_seconds(), Ordering::Relaxed)),
                );
                if beat == 0 {
                    continue;
                }
                let stalled = now.saturating_sub(beat);
                match reported {
                    None if stalled >= HANG_AFTER_SECONDS => {
                        report(
                            Level::Warning,
                            "hang",
                            &format!("The main thread has not answered for {stalled} s"),
                            &format!(
                                "What the app was doing, oldest first (the last line is the most \
                                 likely cause):\n{}\n\nWhere every thread is waiting:\n{}",
                                recent_trail(),
                                crate::hang_stack::capture()
                            ),
                        );
                        reported = Some(now);
                    }
                    Some(since) if stalled < HANG_AFTER_SECONDS => {
                        let seconds = now.saturating_sub(since) + HANG_AFTER_SECONDS;
                        report(
                            Level::Critical,
                            "hang",
                            &format!("Spark stopped responding for about {seconds} seconds"),
                            "The main thread did not answer the watchdog. The lines above this \
                             one in spark-errors.log show what the app was doing.",
                        );
                        reported = None;
                    }
                    _ => {}
                }
            }
        });
    HEARTBEAT.store(unix_seconds(), Ordering::Relaxed);
}

/// Remembers that `what` is about to happen, for the freeze report. Cheap and safe from any thread;
/// never blocks (a busy trail is skipped rather than waited for).
pub fn trail(what: &str) {
    if let Ok(mut trail) = TRAIL.try_lock() {
        if trail.len() == TRAIL_LENGTH {
            trail.pop_front();
        }
        let thread = std::thread::current();
        trail.push_back(format!(
            "{} [{}] {what}",
            &timestamp()[11..],
            thread.name().unwrap_or("unnamed")
        ));
    }
}

fn recent_trail() -> String {
    TRAIL
        .lock()
        .map(|trail| trail.iter().cloned().collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

/// Logs a problem at `level`. `Error` and `Critical` are also shown to the user. Safe to call from
/// any thread.
pub fn report(level: Level, kind: &str, summary: &str, details: &str) {
    let summary = truncate(&redact(summary), MAX_SUMMARY);
    let details = truncate(&redact(details), MAX_DETAILS);
    let key = format!("{kind}:{summary}:{details}");
    let skipped = if level == Level::Critical {
        Some(0)
    } else {
        first_in(&WRITTEN, &key, LOG_REPEAT_SECONDS)
    };
    if let Some(skipped) = skipped {
        let mut text = format!("[{}] {kind}: {summary}", level.label());
        if skipped > 0 {
            text.push_str(&format!(
                " (repeated {skipped} more times since the last entry)"
            ));
        }
        if !details.is_empty() {
            text.push_str(&format!(
                "
{details}"
            ));
        }
        record(&text);
    }
    if !level.notifies() || !may_notify(&format!("{kind}:{summary}")) {
        return;
    }
    let Some(app) = HANDLE.get() else {
        return;
    };
    let report = ErrorReport {
        id: format!("error-{}", COUNTER.fetch_add(1, Ordering::Relaxed)),
        level,
        kind: kind.into(),
        summary,
        details,
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        time: timestamp(),
    };
    ignored(crate::security::emit_shell(app, "error-report", report));
}

/// A failure that was only printed to the console before: it is still printed (development), and now
/// also written to the log as a `Warning` (kind `native`).
pub fn warn(summary: &str) {
    eprintln!("{summary}");
    report(Level::Warning, "native", summary, "");
}

/// Locks `mutex` and returns the guard directly. A `crate::sync::Mutex` never stays poisoned: a
/// lock left by a panic is recovered with its state and logged as a `Warning` with its place in
/// the code (see `crate::sync`).
#[track_caller]
pub fn lock<T: ?Sized>(mutex: &crate::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.guard()
}

/// Use instead of `let _ = result;`: the failure is still ignored, but it is written to the log as
/// a `Warning` with the place in the code that ignored it, so nothing fails invisibly.
#[track_caller]
pub fn ignored<T, E: Debug>(result: Result<T, E>) {
    if let Err(error) = result {
        let place = Location::caller();
        report(
            Level::Warning,
            "ignored",
            &format!("{error:?}"),
            &format!("ignored at {}:{}", place.file(), place.line()),
        );
    }
}

/// One entry of the log file, as the settings screen lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LogEntry {
    pub time: String,
    pub level: String,
    pub text: String,
}

/// How much of the end of the log the settings screen reads.
const READ_TAIL_BYTES: u64 = 192 * 1024;

/// Splits the text of the log into entries. An entry starts with `[time] [LEVEL] ` and continues on the
/// following lines (the details) until the next entry starts. Text before the first entry (a cut line) is dropped.
fn parse_entries(text: &str) -> Vec<LogEntry> {
    let mut entries: Vec<LogEntry> = Vec::new();
    for line in text.lines() {
        if let Some((time, level, first)) = split_entry_start(line) {
            entries.push(LogEntry {
                time: time.into(),
                level: level.into(),
                text: first.into(),
            });
        } else if let Some(last) = entries.last_mut() {
            last.text.push('\n');
            last.text.push_str(line);
        }
    }
    entries
}

fn split_entry_start(line: &str) -> Option<(&str, &str, &str)> {
    let rest = line.strip_prefix('[')?;
    let (time, rest) = rest.split_once("] [")?;
    let (level, text) = rest.split_once("] ")?;
    let known = ["INFO", "WARN", "ERROR", "CRITICAL"];
    (known.contains(&level) && time.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .then_some((time, level, text))
}

/// The latest entries of the log file, newest first, and the file's path. Empty when nothing was logged yet.
pub fn read_recent(max_entries: usize) -> (String, Vec<LogEntry>) {
    use std::io::{Read, Seek, SeekFrom};
    let Some(path) = LOG_FILE.get() else {
        return (String::new(), Vec::new());
    };
    let shown = path.display().to_string();
    let Ok(mut file) = fs::File::open(path) else {
        return (shown, Vec::new());
    };
    let length = file.metadata().map(|m| m.len()).unwrap_or(0);
    if file
        .seek(SeekFrom::Start(length.saturating_sub(READ_TAIL_BYTES)))
        .is_err()
    {
        return (shown, Vec::new());
    }
    let mut bytes = Vec::new();
    if file.read_to_end(&mut bytes).is_err() {
        return (shown, Vec::new());
    }
    let mut entries = parse_entries(&String::from_utf8_lossy(&bytes));
    entries.reverse();
    entries.truncate(max_entries);
    (shown, entries)
}

/// Removes the running marker so the next start knows this run ended normally.
pub fn clean_exit() {
    if let Some(path) = LOG_FILE.get() {
        report(Level::Info, "lifecycle", "Spark closed normally", "");
        ignored(fs::remove_file(path.with_file_name(SESSION_MARKER)));
    }
}

/// `Some(n)` when `key` was not written in the last `within` seconds: write it, and `n` is how many
/// identical entries were skipped since the previous write. `None` when it is a repeat to skip.
fn first_in(
    table: &Mutex<Option<HashMap<String, (u64, u32)>>>,
    key: &str,
    within: u64,
) -> Option<u32> {
    let now = unix_seconds();
    let Ok(mut guard) = table.lock() else {
        return Some(0);
    };
    let seen = guard.get_or_insert_with(HashMap::new);
    if seen.len() >= MAX_REMEMBERED {
        seen.retain(|_, (at, _)| now.saturating_sub(*at) < within);
        if seen.len() >= MAX_REMEMBERED {
            seen.clear();
        }
    }
    match seen.get_mut(key) {
        Some((at, skipped)) if now.saturating_sub(*at) < within => {
            *skipped = skipped.saturating_add(1);
            None
        }
        Some((at, skipped)) => {
            let before = std::mem::take(skipped);
            *at = now;
            Some(before)
        }
        None => {
            seen.insert(key.to_owned(), (now, 0));
            Some(0)
        }
    }
}

/// Applies the repeat window and the per-minute cap to notifications.
fn may_notify(key: &str) -> bool {
    let now = unix_seconds();
    SHOWN.lock().is_ok_and(|mut shown| {
        shown.retain(|(at, _)| now.saturating_sub(*at) < REPEAT_SECONDS);
        if shown.iter().any(|(_, seen)| seen == key) || shown.len() >= MAX_PER_MINUTE {
            false
        } else {
            shown.push((now, key.to_owned()));
            true
        }
    })
}

/// Keeps the user's name and the addresses they visited out of the report: the home folder
/// becomes `~`, and every `http(s)://` address is cut after the host.
pub fn redact(text: &str) -> String {
    let mut text = text.to_owned();
    for home in ["USERPROFILE", "HOME"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
    {
        if home.len() > 3 {
            text = text
                .replace(&home, "~")
                .replace(&home.replace('\\', "/"), "~");
        }
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(start) = ["https://", "http://"]
        .iter()
        .filter_map(|scheme| rest.find(scheme))
        .min()
    {
        let (before, from_scheme) = rest.split_at(start);
        out.push_str(before);
        let end = from_scheme
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ')' | '>' | '<' | ','))
            .unwrap_or(from_scheme.len());
        let (address, after) = from_scheme.split_at(end);
        let scheme_len = address.find("://").map_or(0, |i| i + 3);
        let host_end = address[scheme_len..]
            .find(['/', '?', '#'])
            .map_or(address.len(), |i| scheme_len + i);
        out.push_str(&address[..host_end]);
        if host_end < address.len() {
            out.push_str("/…");
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(max).collect();
    cut.push('…');
    cut
}

/// Appends one line. Failures are ignored: logging must never make a problem worse.
fn record(entry: &str) {
    let Some(path) = LOG_FILE.get() else {
        return;
    };
    if fs::metadata(path).is_ok_and(|m| m.len() > MAX_BYTES) {
        let _ = fs::rename(path, path.with_extension("log.old"));
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "[{}] {entry}", timestamp());
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// `YYYY-MM-DD HH:MM:SS` in UTC, computed here to avoid a date crate for one line.
fn timestamp() -> String {
    format_utc(unix_seconds())
}

fn format_utc(seconds: u64) -> String {
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    // Civil-from-days (Howard Hinnant's algorithm), valid for dates after 1970.
    let z = days as i64 + 719_468;
    let era = z / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_is_split_into_entries_with_their_details() {
        let text = "cut line\n[2026-11-25 10:00:00] [WARN] native: one\n[2026-11-25 10:00:01] [ERROR] interface: two\nmore\ndetails\n[2026-11-25 10:00:02] [INFO] start\n";
        let entries = parse_entries(text);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, "WARN");
        assert_eq!(entries[1].time, "2026-11-25 10:00:01");
        assert_eq!(entries[1].text, "interface: two\nmore\ndetails");
        assert_eq!(entries[2].text, "start");
        assert!(parse_entries("[not a time] [ERROR] x").is_empty());
    }

    #[test]
    fn only_error_and_critical_interrupt_the_user() {
        assert!(!Level::Info.notifies());
        assert!(!Level::Warning.notifies());
        assert!(Level::Error.notifies());
        assert!(Level::Critical.notifies());
        assert_eq!(
            serde_json::to_string(&Level::Critical).unwrap(),
            "\"critical\""
        );
    }

    #[test]
    fn repeats_inside_the_window_are_skipped_and_the_table_stays_bounded() {
        let table = Mutex::new(None);
        assert_eq!(first_in(&table, "a", 60), Some(0));
        assert_eq!(first_in(&table, "a", 60), None);
        assert_eq!(first_in(&table, "b", 60), Some(0));
        for n in 0..MAX_REMEMBERED * 2 {
            first_in(&table, &n.to_string(), 60);
        }
        assert!(table.lock().unwrap().as_ref().unwrap().len() <= MAX_REMEMBERED);
    }

    #[test]
    fn skipped_repeats_are_counted_and_reported_with_the_next_entry() {
        let table = Mutex::new(None);
        assert_eq!(first_in(&table, "a", 60), Some(0));
        assert_eq!(first_in(&table, "a", 60), None);
        assert_eq!(first_in(&table, "a", 60), None);
        // With a zero window the entry is written again and reports what was skipped.
        assert_eq!(first_in(&table, "a", 0), Some(2));
        assert_eq!(first_in(&table, "a", 0), Some(0));
    }

    #[test]
    fn the_trail_keeps_only_the_latest_entries_in_order() {
        for n in 0..TRAIL_LENGTH + 5 {
            trail(&format!("step {n}"));
        }
        let text = recent_trail();
        assert!(text.lines().count() <= TRAIL_LENGTH);
        assert!(text
            .lines()
            .last()
            .is_some_and(|line| line.ends_with(&format!("step {}", TRAIL_LENGTH + 4))));
    }

    #[test]
    fn ignoring_a_success_or_a_failure_never_panics() {
        ignored(Ok::<_, String>(1));
        ignored(Err::<(), _>("boom"));
    }

    #[test]
    fn a_lock_poisoned_by_a_panic_is_recovered_with_its_state() {
        let mutex = std::sync::Arc::new(crate::sync::Mutex::new(1));
        let poisoner = mutex.clone();
        let outcome = std::thread::spawn(move || {
            *lock(&poisoner) = 2;
            let _guard = lock(&poisoner);
            panic!("poison the lock");
        })
        .join();
        assert!(outcome.is_err());
        assert_eq!(*lock(&mutex), 2, "the state left by the panic is kept");
        *lock(&mutex) = 3;
        assert_eq!(*lock(&mutex), 3);
    }

    #[test]
    fn redact_cuts_addresses_after_the_host_and_hides_the_home_folder() {
        assert_eq!(
            redact("failed https://example.test/a/b?token=1#x, then http://h.test"),
            "failed https://example.test/…, then http://h.test"
        );
        assert_eq!(redact("no address"), "no address");
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            if home.len() > 3 {
                assert_eq!(redact(&format!("{home}/x/y.rs")), "~/x/y.rs");
            }
        }
    }

    #[test]
    fn truncate_keeps_short_text_and_marks_cuts() {
        assert_eq!(truncate("abc", 3), "abc");
        assert_eq!(truncate("abcd", 3), "abc…");
    }

    #[test]
    fn timestamps_are_utc_calendar_dates() {
        assert_eq!(format_utc(0), "1970-01-01 00:00:00");
        assert_eq!(format_utc(951_782_400), "2000-02-29 00:00:00");
        assert_eq!(format_utc(1_794_847_445), "2026-11-16 16:44:05");
    }
}
