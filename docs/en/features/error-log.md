# Error log and error notifications

[Português](../../pt-BR/features/error-log.md)

Spark tells the user when something goes wrong and keeps a local log, so bugs can be reported with the details already collected. Spark never uploads anything by itself: the user chooses to send an error.

## Notification

When an error happens, a card appears in the **bottom-right corner** of the browser window (up to three at a time, the rest wait). It stays until dismissed and has two buttons:

- **View error** shows the details inside the card.
- **Send error** opens a new Spark tab on the project's **new issue** page (`https://github.com/BadTonho/SparkBrowser/issues/new`) with the title and body already filled in: Spark version, system, time, type, summary and details (cut to fit the address limit). The user reviews it and submits on GitHub. If the page cannot be opened, the card says so.

The same error is shown at most once a minute and at most five cards appear per minute, so a loop cannot flood the screen.

## Levels

Every entry has one of four levels. Only the two serious ones interrupt the user with a card; the other two are only written to the log.

| Level | Meaning | Examples | Card |
| --- | --- | --- | --- |
| `info` | Notable events that are not problems | start, normal close | no |
| `warning` | A failure Spark ignored and recovered from | a discarded `Result` in Rust, an empty `.catch` in the interface, the start of a freeze | no |
| `error` | Something failed and the user may notice | native errors, script errors | yes |
| `critical` | The app itself is in trouble | a panic, a freeze that ended, a run that did not close normally | yes, in a stronger red |

## What counts as an error

| Type | Source |
| --- | --- |
| `panic` | An unexpected internal error in the Rust code (any thread) |
| `hang` | The main thread did not answer a watchdog for 10 seconds; reported when it answers again, with the duration |
| `native` | Any error Spark already showed through the `browser-error` event (downloads, privacy, tab operations, and so on). Blocks by the shield ("Navigation blocked") are intentional and are not reported |
| `interface` | A script error or unhandled rejection in the browser interface (shell, settings, tools, downloads) |
| `new-tab` | A script error or unhandled rejection in the new-tab page (limited to 3 per page load) |
| `shutdown` | Saving or closing Spark failed (save error, installer or restart). The log has the exact failure, the purpose and the phase; the closing dialog shows only a friendly message |
| `unclean-exit` | The previous run left its `session.running` marker behind: a crash, a force-closed freeze or a power loss. Shown a few seconds after start |

### Context of a freeze

A freeze has no error message, so its `hang` entry carries a **trail**: the last 40 things the app was asked to do (every interface command and every popover open or close), each with time and thread. The last line is the most likely cause. On Windows the entry also lists **where every thread is waiting** (its call stack), which shows a deadlock directly: two threads each waiting for something the other holds. Function names appear when the `.pdb` file is next to the executable (development builds); a released build shows module offsets. The code is in [`src-tauri/src/hang_stack.rs`](../../../src-tauri/src/hang_stack.rs).

### Failed commands and the console

Every interface command that fails is logged as a `warning` (`Command "name" failed: …`), even when the caller handles the failure. Anything the interface prints with `console.error` is logged as a `warning` too.

### Failures that were ignored

Code that used to throw an error away now logs it as a `warning` with the place that ignored it:

- In Rust, `let _ = result;` became `crate::error_log::ignored(result);`, which logs the error and the file and line. Use it for every new `Result` that is deliberately ignored.
- In the interface, an empty `.catch(() => {})` became `.catch(logIgnored)` (`src/lib/services/ignored-error.ts`). Promise queues that only keep a chain alive keep their empty catch, because the error belongs to the caller.

Everything that used to be printed only to the console (`eprintln!`) is now logged as a `warning` too (`crate::error_log::warn`), and the new-tab page logs its ignored failures through `window.__sparkLogIgnored`. Not covered yet: Rust code that only compiles on Linux was converted without being compiled here, the scripts injected into Google Drive pages (`gdrive_bridge.js`) still have best-effort empty catches, and a few Rust places turn an error into a plain "failed" without keeping its text.

## Log file

`logs/spark-errors.log` inside the Spark data directory (on Windows, `%APPDATA%\org.spark.browser\logs`). Every entry, each start and each normal close is appended there, one line per entry with its level (`[INFO]`, `[WARN]`, `[ERROR]`, `[CRITICAL]`). The same entry is written at most once every 10 seconds, except `critical`; the next entry written for it says how many repeats were skipped (`repeated N more times since the last entry`). At 512 KB it is renamed to `spark-errors.log.old`, so it never uses more than about 1 MB. Private windows do not write to it or show these cards.

## Privacy

Reports are redacted before they are logged or shown: the user's home folder becomes `~`, and any `http(s)://` address is cut after the host (`https://example.test/…`). Summaries are limited to 300 characters and details to 6,000.

## Code

- [`src-tauri/src/error_log.rs`](../../../src-tauri/src/error_log.rs): log, redaction, rate limit, panic hook, hang watchdog, run marker. `install` runs in `run()` before any window opens, `watch` in the application `setup`, `clean_exit` on `RunEvent::Exit`.
- [`src-tauri/src/commands/error_report.rs`](../../../src-tauri/src/commands/error_report.rs): the `report_frontend_error` command used by the interface.
- [`src-tauri/src/window/new_tab/bridge.js`](../../../src-tauri/src/window/new_tab/bridge.js): sends new-tab page errors through the authenticated `/error` bridge path.
- [`src/lib/components/status/ErrorReportToast.svelte`](../../../src/lib/components/status/ErrorReportToast.svelte) and [`src/lib/services/error-report.js`](../../../src/lib/services/error-report.js): the card and the issue address.
