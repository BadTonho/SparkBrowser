# AppContainer isolation (planned)

Status: **planned, not implemented**. Identified as the highest-risk hardening idea during the isolation-layer review; the current decision is to document the design first and put it on the roadmap instead of shipping an experimental launcher.

## Goal

Run the whole browser host inside a Windows AppContainer, the same OS-level containment model Edge and Chrome use for their own processes. A misbehaving renderer would then be constrained twice: by the WebView2 renderer sandbox and by the host process token.

## Proposed architecture

1. A small launcher executable creates an AppContainer security token (untrusted integrity, one profile SID plus the Internet and media capabilities WebView2 requires), then starts the real browser binary with that token and drops its own privileges.
2. The launcher pre-installs access-control entries for the AppContainer SID on the profile directory and the downloads directory, because AppContainer processes cannot read arbitrary user paths.
3. The updater either restarts through the launcher (so containment does not silently disappear after an update) or the launcher re-derives the token before relaunching; both paths must be validated.
4. Disabled by default behind a settings entry and a native smoke test, so ordinary installs keep the exact current behavior.

## Known conflicts that need resolution first

- **Updater**: `tauri-plugin-updater` relaunches the current executable directly; that path must go through the launcher or containment evaporates after the first update.
- **File access**: SQLite, downloads, drag-and-drop targets and the WebView2 runtime state all need explicit ACLs; failures would show up as hard-to-debug permission errors.
- **Auxiliary flows**: schema-less protocol handler registration, shell integration and updater signatures (see `src-tauri/tests/updater_signatures.rs`) must be re-validated under the restricted token.
- **Debugging**: developer tools on the shell and the native test harness must remain usable in development builds, where the option stays off.

## Acceptance criteria before implementing

- The native test suite (`npm run test:native`) passed twice in a row with isolation enabled: cold start, update restart, download and save, drag-and-drop, workspace isolation on and off.
- Killing the launcher, killing WebView2 processes and toggling the option mid-session never leave the browser running outside the container.
- The feature has an explicit opt-in, an in-product explanation of its costs, and a one-command rollback.

Until then, no launcher ships. The application boundaries already keep content WebViews without application commands, host objects or persistent inter-tab surface; AppContainer would harden the last layer between a compromised renderer process and the operating system.
