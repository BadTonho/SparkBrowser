# Workspace content isolation

Status: **implemented, opt-in and disabled by default**. A Websites/cookies model built on top of the application boundaries keeps one shared browsing profile; this feature adds optional per-workspace separation.

## What it changes

Settings → Workspaces exposes **“Isolate workspace content”**. While it is enabled, every content WebView receives its own WebView2 user data folder derived from the owning workspace id:

- Cookies, `localStorage`/`IndexedDB` and HTTP caches become workspace-scoped; the same site active in two workspaces keeps two independent sessions.
- The WebView2 runtime keeps a separate browser process per folder, which also adds a process boundary between workspaces.
- The shell, internal panels and PiP windows keep the default application profile; only tab content is scoped.

## Lifecycle

- The preference is stored durably in the `app_preferences` SQLite table and is loaded before the session is restored, so the restore already runs with the correct folders.
- Enabling or disabling the setting rebuilds content WebViews; pages lose their live state. This matches the privacy-shield contract and is announced by the shell.
- Moving a tab to another workspace drops its WebView first, so no document keeps writing into the previous workspace's folder; the next sync recreates it inside the target folder.
- Deleting a workspace removes its data folder after document teardown (best effort; locked files are skipped).

## Folder naming

Only lowercase ASCII letters, digits, `-` and `_` up to 48 bytes, without a trailing `-` or a Windows device name, retain their legacy folder name. Every other id, including mixed-case ids, hashes in full to a 64-hex-digit `h-…` folder (SHA-256). Distinct case-sensitive database ids therefore remain distinct on both Windows and Linux. Existing lowercase workspace folders remain unchanged; unusual legacy mixed-case/device-name ids receive a separate profile and may require signing in again. Folders live under the application's local data directory in `webview-workspaces/<name>`.

## Costs and limits

- Each workspace with visible tabs keeps one more WebView2 browser process; measure memory on low-RAM devices before enabling.
- Restoring a workspace session does not restore page-local history of tabs that moved between workspaces; global history, sessions and the privacy shield are unaffected.
- A renderer exploit inside one workspace can still read that workspace's cookies. Full containment of spoofed renderer capabilities is covered separately in [AppContainer isolation](../security/app-container-isolation.md).

Synced with the interface texts; see `tests/security-hardening.test.mjs` for the enforced invariants.
