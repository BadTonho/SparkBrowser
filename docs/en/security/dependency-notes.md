# Dependency and platform notes

[Português](../../pt-BR/security/dependency-notes.md)

## JavaScript security backport

`@sveltejs/vite-plugin-svelte` depends on `deepmerge 4.3.1`, affected by [GHSA-gcqx-w48m-w2h4](https://github.com/advisories/GHSA-gcqx-w48m-w2h4). At the time of this change, the advisory lists no patched upstream release, and the latest Svelte plugin still depends on the affected package.

The npm override resolves `deepmerge` to `tools/vendor/deepmerge`, a private fork named `@spark/deepmerge`. It contains the upstream 4.3.1 bundled CommonJS implementation and its MIT license. The only behavior change uses `Object.defineProperty` for destination writes from **both target and source** so an enumerable `__proto__` cannot invoke the inherited prototype setter. The fork has an explicit patched version and is never published.

`tests/deepmerge-security.test.mjs` resolves the dependency from the actual Svelte plugin and verifies the regression plus arrays, symbols, nested options, and custom merge behavior. Replacing this fork requires equivalent regression checks and a fresh npm audit; do not remove it solely to reduce vendor files.

## Rust and Linux upstream warnings

Spark bundles SQLite through `rusqlite 0.41`. The locked `libsqlite3-sys 0.39.1` includes the fix for [CVE-2026-119375](https://rustsec.org/advisories/RUSTSEC-2026-0459.html) on Windows and Linux.

`cargo audit --no-fetch` uses the local advisory database. It does not certify that the database is current. The current Tauri dependency tree still includes Linux GTK3 crates marked unmaintained and `glib 0.18.5`, affected by [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), plus an unmaintained `proc-macro-error` dependency.

The affected glib iterator is fixed upstream in `glib >=0.20`, outside the GTK3 binding stack's compatible dependency range. Forcing that version into the existing graph is not a supported upgrade. These warnings remain visible: no audit exceptions hide them. A supported upstream Tauri/GTK migration is still required before claiming that the Linux dependency warnings are resolved.

## Platform validation

Local automated validation targets Windows, including an isolated release WebDriver suite. The release workflow also builds Linux packages and runs Rust tests; native WebView behavior and installers require separate checks. The multi-WebView API uses Tauri's `unstable` feature and system engines differ between platforms. No cross-platform runtime guarantee or fixed executable-size claim is made.

## Local data

User databases live in the operating system application-data directory. Database files, SQLite WAL/SHM sidecars, credentials, build outputs, and `.cache/` are excluded from Git. Before committing, inspect status and both staged and unstaged diffs; ignore rules do not protect a file that is already tracked.
