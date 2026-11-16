# Error log and error notifications

[Português](../../pt-BR/features/error-log.md)

Spark tells the user when something goes wrong and keeps a local log, so bugs can be reported with the details already collected. Spark never uploads anything by itself: the user chooses to send an error.

## Notification

When an error happens, a card appears in the **bottom-right corner** of the browser window (up to three at a time, the rest wait). It stays until dismissed and has two buttons:

- **View error** shows the details inside the card.
- **Send error** opens a new Spark tab on the project's **new issue** page (`https://github.com/BadTonho/SparkBrowser/issues/new`) with the title and body already filled in: Spark version, system, time, type, summary and details (cut to fit the address limit). The user reviews it and submits on GitHub. If the page cannot be opened, the card says so.

The same error is shown at most once a minute and at most five cards appear per minute, so a loop cannot flood the screen.

## What counts as an error

| Type | Source |
| --- | --- |
| `panic` | An unexpected internal error in the Rust code (any thread) |
| `hang` | The main thread did not answer a watchdog for 10 seconds; reported when it answers again, with the duration |
| `native` | Any error Spark already showed through the `browser-error` event (downloads, privacy, tab operations, and so on). Blocks by the shield ("Navigation blocked") are intentional and are not reported |
| `interface` | A script error or unhandled rejection in the browser interface (shell, settings, tools, downloads) |
| `new-tab` | A script error or unhandled rejection in the new-tab page (limited to 3 per page load) |
| `unclean-exit` | The previous run left its `session.running` marker behind: a crash, a force-closed freeze or a power loss. Shown a few seconds after start |

Errors that Spark handles silently without ever raising an error event (a `let _ = ...` in Rust) are not reported.

## Log file

`logs/spark-errors.log` inside the Spark data directory (on Windows, `%APPDATA%\org.spark.browser\logs`). Every report, each start and each normal close is appended there. At 512 KB it is renamed to `spark-errors.log.old`, so it never uses more than about 1 MB. Private windows do not write to it or show these cards.

## Privacy

Reports are redacted before they are logged or shown: the user's home folder becomes `~`, and any `http(s)://` address is cut after the host (`https://example.test/…`). Summaries are limited to 300 characters and details to 6,000.

## Code

- [`src-tauri/src/error_log.rs`](../../../src-tauri/src/error_log.rs): log, redaction, rate limit, panic hook, hang watchdog, run marker. `install` runs in `run()` before any window opens, `watch` in the application `setup`, `clean_exit` on `RunEvent::Exit`.
- [`src-tauri/src/commands/error_report.rs`](../../../src-tauri/src/commands/error_report.rs): the `report_frontend_error` command used by the interface.
- [`src-tauri/src/window/new_tab/bridge.js`](../../../src-tauri/src/window/new_tab/bridge.js): sends new-tab page errors through the authenticated `/error` bridge path.
- [`src/lib/components/status/ErrorReportToast.svelte`](../../../src/lib/components/status/ErrorReportToast.svelte) and [`src/lib/services/error-report.js`](../../../src/lib/services/error-report.js): the card and the issue address.
