//! Local error log and error notifications.
//!
//! Every problem that reaches [`report`] (a panic, a window that stopped responding, a native
//! error, a script error from the interface or the new-tab page, or a run that did not end
//! cleanly) is appended to `<data directory>/logs/spark-errors.log` and sent to the browser shell,
//! which shows a notification with buttons to view the error and to send it as a GitHub issue.
//!
//! Nothing is sent by Spark itself. Entries are redacted first: the user's home folder becomes `~`
//! and page addresses lose their path and query.
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
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

/// Same error shown at most once in this many seconds, so a loop cannot flood the screen.
const REPEAT_SECONDS: u64 = 60;
/// Notifications per minute, across all errors.
const MAX_PER_MINUTE: usize = 5;
const MAX_SUMMARY: usize = 300;
const MAX_DETAILS: usize = 6_000;
const SESSION_MARKER: &str = "session.running";

static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();
static HANDLE: OnceLock<AppHandle> = OnceLock::new();
static HEARTBEAT: AtomicU64 = AtomicU64::new(0);
static COUNTER: AtomicU64 = AtomicU64::new(0);
static RECENT: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());
/// Set when the previous run left its marker behind, which means it did not close normally.
static PREVIOUS_RUN_ENDED_BADLY: OnceLock<bool> = OnceLock::new();

/// What the shell shows in the error notification.
#[derive(Clone, Serialize)]
pub struct ErrorReport {
    pub id: String,
    /// `panic`, `hang`, `native`, `interface`, `new-tab` or `unclean-exit`.
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

/// Installs the panic hook. Call once, as early as the data directory is known. The previous hook
/// still runs, so the console output in development builds is unchanged.
pub fn install(data_directory: &Path) {
    let directory = directory(data_directory);
    if fs::create_dir_all(&directory).is_err() || LOG_FILE.set(directory.join(FILE_NAME)).is_err() {
        return;
    }
    let marker = directory.join(SESSION_MARKER);
    let _ = PREVIOUS_RUN_ENDED_BADLY.set(marker.exists());
    let _ = fs::write(&marker, unix_seconds().to_string());
    record(&format!(
        "---- Spark {} started ----",
        env!("CARGO_PKG_VERSION")
    ));
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
        let details = format!(
            "at {location}
{backtrace}"
        );
        let _ = std::thread::Builder::new()
            .name("error-log-panic".into())
            .spawn(move || report("panic", &summary, &details))
            .map(|handle| handle.join());
        previous(info);
    }));
}

/// Watches the main thread: every second it asks the event loop to update a counter, and a counter
/// that stops moving for `HANG_AFTER_SECONDS` is written to the log, followed by a note when the
/// window answers again. That tells a freeze apart from a crash and shows how long it lasted.
pub fn watch(app: &AppHandle) {
    if LOG_FILE.get().is_none() {
        return;
    }
    let _ = HANDLE.set(app.clone());
    if PREVIOUS_RUN_ENDED_BADLY.get().copied().unwrap_or(false) {
        // Wait until the interface is listening before telling the user.
        let _ = std::thread::Builder::new()
            .name("error-log-previous-run".into())
            .spawn(|| {
                std::thread::sleep(Duration::from_secs(6));
                report(
                    "unclean-exit",
                    "Spark did not close normally the last time it ran",
                    "The previous run left its running marker behind, so it ended without a clean                      shutdown: a crash, a freeze that was force-closed, or the computer losing power.",
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
                let _ =
                    app.run_on_main_thread(|| HEARTBEAT.store(unix_seconds(), Ordering::Relaxed));
                if beat == 0 {
                    continue;
                }
                let stalled = now.saturating_sub(beat);
                match reported {
                    None if stalled >= HANG_AFTER_SECONDS => {
                        record(&format!(
                            "HANG: the main thread has not answered for {stalled} s"
                        ));
                        reported = Some(now);
                    }
                    Some(since) if stalled < HANG_AFTER_SECONDS => {
                        let seconds = now.saturating_sub(since) + HANG_AFTER_SECONDS;
                        report(
                            "hang",
                            &format!("Spark stopped responding for about {seconds} seconds"),
                            "The main thread did not answer the watchdog. See the lines above                              this one in spark-errors.log for what the app was doing.",
                        );
                        reported = None;
                    }
                    _ => {}
                }
            }
        });
    HEARTBEAT.store(unix_seconds(), Ordering::Relaxed);
}

/// Logs the problem and tells the shell to show the notification. Safe to call from any thread.
pub fn report(kind: &str, summary: &str, details: &str) {
    let summary = truncate(&redact(summary), MAX_SUMMARY);
    let details = truncate(&redact(details), MAX_DETAILS);
    record(&format!(
        "{} {kind}: {summary}
{details}",
        if kind == "panic" { "PANIC" } else { "ERROR" }
    ));
    let now = unix_seconds();
    let allowed = RECENT.lock().is_ok_and(|mut recent| {
        recent.retain(|(at, _)| now.saturating_sub(*at) < REPEAT_SECONDS);
        let key = format!("{kind}:{summary}");
        if recent.iter().any(|(_, seen)| *seen == key) || recent.len() >= MAX_PER_MINUTE {
            false
        } else {
            recent.push((now, key));
            true
        }
    });
    let Some(app) = HANDLE.get().filter(|_| allowed) else {
        return;
    };
    let report = ErrorReport {
        id: format!("error-{}", COUNTER.fetch_add(1, Ordering::Relaxed)),
        kind: kind.into(),
        summary,
        details,
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        time: timestamp(),
    };
    let _ = crate::security::emit_shell(app, "error-report", report);
}

/// Removes the running marker so the next start knows this run ended normally.
pub fn clean_exit() {
    if let Some(path) = LOG_FILE.get() {
        record("---- Spark closed normally ----");
        let _ = fs::remove_file(path.with_file_name(SESSION_MARKER));
    }
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

/// Appends one entry. Failures are ignored: logging must never make a problem worse.
pub fn record(entry: &str) {
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
