# Release channels and updates

Open **Settings → About & Updates** to see the running version, choose **Stable** or **Beta**, check for updates and read release notes. Windows x64 release installations check once when opened. Startup failures appear in this panel without interrupting browsing. Development and isolated native builds and other platforms can save a channel preference, but cannot check or install updates. Browser preview cannot save the preference.

Open **Settings → Changelog** to read the release notes of every version, newest first, in the interface language (English if a version has no translation). The newest version starts expanded and the running version is marked. The notes ship inside the app from the root `changelog/` folder, so they work offline and need no network request, and they are shown as plain text. The category is also found by searching the settings for "changelog", "release notes" or "novidades".

Stable accepts published tags such as `v1.0.0`. Beta also accepts `v1.1.0-beta.1`, marked as a GitHub pre-release, and newer stable versions. SemVer ordering places `beta.10` after `beta.2`, and `1.1.0` after `1.1.0-beta.10`. Drafts, unsupported tags and inconsistent pre-release flags are excluded. Pagination errors and rate limits are reported instead of claiming the installation is current.

Both channels share the installation and local data. The first launch defaults to the installed build's channel; later launches retain the preference. Switching from a newer beta to Stable waits for a newer stable release and never downgrades the application or database. No cloud synchronization or GitHub login is used.

An available update shows a dismissible notice. **Download update** downloads and verifies without installing. **Install and restart** requires confirmation. Edited forms and playing media require an additional confirmation; queued or active downloads must finish or be cancelled. Pending notes must save successfully, navigation writes are drained and the session is committed before installation. Errors retain the running installation and allow retry. Forms, scroll and media state are lost on restart.

Rust reads public releases from `BadTonho/SparkBrowser`, validates the tag, manifest and installer URL, and uses the official Tauri updater. Packages need a valid signature bound to the announced version. Installer bytes remain in memory until installation, channel change or exit; restarting requires downloading again. Only the local `main` WebView can invoke application update commands and receive update-state events. Direct updater-plugin IPC permissions are not granted.

See [local signed releases](../contributing/releases.md). Versions without this updater need one initial manual installation.
