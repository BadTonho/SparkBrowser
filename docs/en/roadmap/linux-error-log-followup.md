# Linux follow-up: error log

[Português](../../pt-BR/roadmap/linux-error-log-followup.md)

Status: **pending**, to be done on a Linux machine.

## Why

The error log ([error-log.md](../features/error-log.md)) records a failure that Rust code used to throw away (`let _ = result;`) when that code calls `crate::error_log::ignored(result)` instead. The conversion was done on Windows with `cargo check`, so only code that compiles on Windows could be converted and verified. Code that only compiles on Linux (`#[cfg(target_os = "linux")]`, `#[cfg(not(windows))]`, `#[cfg(unix)]`, files such as `*_linux.rs`) was left unchanged to avoid breaking the Linux build.

## What to do

1. On Linux, install the build dependencies from the [Linux support guide](../features/linux-support.md) and make sure `cargo check --lib` passes in `src-tauri` before changing anything.
2. For each site listed below, replace `let _ = EXPR;` with `crate::error_log::ignored(EXPR);` (the statement can span several lines; keep the closing `);`).
3. Run `cargo check --lib` on Linux. A site that does not compile is one of:
   - `EXPR` is not a `Result` (it returns `bool`, `Option`, `()` or a `JoinHandle`): put `let _ = ` back.
   - The error type does not implement `Debug` (`` `T` doesn't implement `Debug` ``): put `let _ = ` back, or map the error first with `.map_err(|e| e.to_string())`.
4. Run `cargo fmt`, `cargo test --lib` and `npm test`.
5. Check that the Windows build still compiles (`cargo check --lib` on Windows, or in CI).
6. Add a line to the changelog of the version in progress (`Improvements`, English and Portuguese): errors that Linux used to ignore silently are now logged.
7. Commit, for example `feat(error-log): log ignored Results in Linux-only code`.

Sites that sit inside another converted statement (a closure inside `ignored(...)`) can stay as they are.

## Sites to check

Line numbers are from commit `b1c365f`. The list is every remaining `let _ = call(...)` outside tests; some of them compile on Windows and were left because they are not a `Result` or have no `Debug` error, so expect to put a few back.

- [`backup/credentials.rs`](../../../src-tauri/src/backup/credentials.rs): 75, 179, 180, 192, 193
- [`backup/sync.rs`](../../../src-tauri/src/backup/sync.rs): 297, 308
- [`commands/capture_probe.rs`](../../../src-tauri/src/commands/capture_probe.rs): 172
- [`commands/dark_mode_probe.rs`](../../../src-tauri/src/commands/dark_mode_probe.rs): 73
- [`commands/media.rs`](../../../src-tauri/src/commands/media.rs): 16, 66
- [`commands/minimum_font_probe.rs`](../../../src-tauri/src/commands/minimum_font_probe.rs): 25
- [`commands/privacy.rs`](../../../src-tauri/src/commands/privacy.rs): 273, 279
- [`commands/site_permissions.rs`](../../../src-tauri/src/commands/site_permissions.rs): 41, 61
- [`commands/updates.rs`](../../../src-tauri/src/commands/updates.rs): 41, 64, 90
- [`commands/whatsapp.rs`](../../../src-tauri/src/commands/whatsapp.rs): 32
- [`downloads/native.rs`](../../../src-tauri/src/downloads/native.rs): 14
- [`downloads/native_fallback.rs`](../../../src-tauri/src/downloads/native_fallback.rs): 21, 105, 122, 133, 135
- [`downloads/native_windows.rs`](../../../src-tauri/src/downloads/native_windows.rs): 565, 569
- [`extensions/windows.rs`](../../../src-tauri/src/extensions/windows.rs): 15
- [`media/linux.rs`](../../../src-tauri/src/media/linux.rs): 27
- [`media/mod.rs`](../../../src-tauri/src/media/mod.rs): 224
- [`media/pip.rs`](../../../src-tauri/src/media/pip.rs): 133
- [`media/protocol_linux.rs`](../../../src-tauri/src/media/protocol_linux.rs): 109
- [`media/visualizer/windows.rs`](../../../src-tauri/src/media/visualizer/windows.rs): 249, 252, 255
- [`media/volume.rs`](../../../src-tauri/src/media/volume.rs): 125, 222
- [`privacy/content_filter_linux.rs`](../../../src-tauri/src/privacy/content_filter_linux.rs): 39, 47
- [`privacy/filter_lists.rs`](../../../src-tauri/src/privacy/filter_lists.rs): 125
- [`security/mod.rs`](../../../src-tauri/src/security/mod.rs): 103
- [`site_permissions/linux.rs`](../../../src-tauri/src/site_permissions/linux.rs): 38, 48, 218
- [`site_permissions/mod.rs`](../../../src-tauri/src/site_permissions/mod.rs): 89, 92
- [`site_permissions/profile.rs`](../../../src-tauri/src/site_permissions/profile.rs): 15
- [`updates/mod.rs`](../../../src-tauri/src/updates/mod.rs): 55
- [`updates/service.rs`](../../../src-tauri/src/updates/service.rs): 213
- [`webview_runtime/mod.rs`](../../../src-tauri/src/webview_runtime/mod.rs): 91
- [`window/browser_windows.rs`](../../../src-tauri/src/window/browser_windows.rs): 36
- [`window/caret_browsing.rs`](../../../src-tauri/src/window/caret_browsing.rs): 22
- [`window/favicon/linux.rs`](../../../src-tauri/src/window/favicon/linux.rs): 66
- [`window/new_tab/widgets/cookies.rs`](../../../src-tauri/src/window/new_tab/widgets/cookies.rs): 124, 128
- [`window/new_tab/widgets/spotify/platform.rs`](../../../src-tauri/src/window/new_tab/widgets/spotify/platform.rs): 100
- [`window/overlay_host.rs`](../../../src-tauri/src/window/overlay_host.rs): 105
- [`window/settings_tab.rs`](../../../src-tauri/src/window/settings_tab.rs): 234
- [`window/tab_drag_preview/surface.rs`](../../../src-tauri/src/window/tab_drag_preview/surface.rs): 58, 60
- [`window/tab_memory.rs`](../../../src-tauri/src/window/tab_memory.rs): 112
- [`window/vibrancy.rs`](../../../src-tauri/src/window/vibrancy.rs): 20

Known to fail on Windows already: `extensions/windows.rs`, `site_permissions/profile.rs` (no `Debug` on the error) and `window/overlay_host.rs` (not a `Result`).

## Also not covered

- Scripts of the new-tab page (`public/new-tab/*.js`) still have empty `catch` blocks. They cannot call the log directly; they would have to send an `error` message through the bridge, like unhandled errors already do.
- The notification only shows `error` and `critical`. If you want a setting to show warnings too, it is a small change in `error_log.rs` (`Level::notifies`).
