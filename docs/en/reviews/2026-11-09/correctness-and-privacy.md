# Correctness and privacy fixes

The seven findings from the project review have code changes and regression coverage.

- WebDAV credentials use Windows Credential Manager or Linux Secret Service through `secret-tool`. Legacy plaintext is removed using SQLite secure deletion and a truncating WAL checkpoint. A failed system-store migration retains the credential only in process memory. Existing external database copies remain outside this migration.
- Google Drive synchronization binds the explicitly selected persistent profile and account index. Transfers verify the detected email, reject identity changes and do not fall back to disposable tabs or unrelated cookie profiles. Existing users must select their Google account once after upgrading.
- Backup import preserves disposable tabs, local order, selection, split membership and resident document state. The import transaction preserves existing account-container identities; imported tabs still use the default identity as documented.
- Workspace folder names preserve ordinary lowercase legacy names and hash mixed-case, unsafe and Windows device-name identifiers without case collisions. Unusual old identifiers may require a new website login.
- RSS/Atom parsing accepts CDATA titles and links, retains nested text and selects article alternatives instead of self/API/enclosure links.
- Rust formatting and strict Clippy checks pass. Tab-profile and download-registration options replace long positional argument lists.

The additional improvements are included: regression tests exercise behavior, feed refreshes reuse an HTTP client with four workers and per-feed deduplication, and the Drive integration separates authentication helpers, cookie access, HTTP transfer and session binding into `src-tauri/src/backup/gdrive/`.

## Verification

- `npm run validate`: JavaScript tests, Svelte/TypeScript checks, Rust formatting, strict Clippy, Rust tests, frontend build and repository security checks.
- Native Windows scenarios: `backup-identities` (including an immediate forced restart), `backup` (including preference retry), and `feeds` (including CDATA, Atom alternatives, malformed XML and restart persistence).
- Credential migration tests inspect both an on-disk SQLite database and its WAL for the artificial credential after migration. Credential-store unit tests use an isolated in-memory adapter and never access personal credentials.

Linux native behavior, the Linux Secret Service adapter and transfers against a real signed-in Google account were not exercised in this environment. The Google integration still depends on Google's first-party web endpoints; native fixture results do not prove compatibility with future server changes. See the [backup guide](../../features/backup.md), [workspace isolation guide](../../features/workspace-isolation.md), and [feed reader guide](../../features/feed-reader.md) for usage and platform requirements.
