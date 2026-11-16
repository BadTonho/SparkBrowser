# Release channels and updates

Open **Settings → About & Updates** to see the running version, choose **Stable** or **Beta**, check for updates and read release notes. Windows x64 release installations check once when opened. Startup failures appear in this panel without interrupting browsing. Development and isolated native builds and other platforms can save a channel preference, but cannot check or install updates. Browser preview cannot save the preference.

Open **Settings → Changelog** to read the release notes of every version, newest first, in the interface language (English if a version has no translation). The newest version starts expanded and the running version is marked. The notes ship inside the app from the root `changelog/` folder, so they work offline and need no network request, and they are shown as plain text. The category is also found by searching the settings for "changelog", "release notes" or "novidades".

Stable accepts published tags such as `v1.0.0`. Beta also accepts `v1.1.0-beta.1`, marked as a GitHub pre-release, and newer stable versions. SemVer ordering places `beta.10` after `beta.2`, and `1.1.0` after `1.1.0-beta.10`. Drafts, unsupported tags and inconsistent pre-release flags are excluded. Pagination errors and rate limits are reported instead of claiming the installation is current.

Both channels share the installation and local data. The first launch defaults to the installed build's channel; later launches retain the preference. Switching from a newer beta to Stable waits for a newer stable release and never downgrades the application or database. No cloud synchronization or GitHub login is used.

## Automatic updates

**Settings → About & Updates → Update automatically** is **on by default** (also for existing profiles). When the check at startup finds an update, Spark downloads and verifies it in the background without asking. Once it is ready, the notice at the top of the window and the settings show **Restart to update**. Press it to install now; the usual close review still appears if a page has an edited form, playing media or a download. If nobody presses it, the update installs the next time Spark opens.

To make that possible, the verified installer is also saved in the profile folder (`updates/staged`) together with the release manifest. At the next start, before the previous session is restored, Spark checks that the file is still newer than the running version, belongs to the current channel and has a valid signature, deletes it, and runs the installer, which restarts Spark on the new version. The files are deleted before the installer runs, so an installer that fails is never retried at every start, and a signature mismatch or an older version is ignored. Private windows never install. Because the run marker is not touched yet, the restart is not treated as a crash and the session is restored normally.

Turning the option **off** returns to the manual flow described below and deletes a saved update. Changing the channel also deletes it. The checkbox is a local preference, not part of backups. Updates are still checked when Spark opens (and with **Check for updates**), so a browser kept open for days finds the next update at its next start.

An available update shows a dismissible notice. **Download update** downloads and verifies without installing. **Install and restart** requires confirmation. Edited forms and playing media require an additional confirmation; queued or active downloads must finish or be cancelled. Pending notes must save successfully, navigation writes are drained and the session is committed before installation. Errors retain the running installation and allow retry. Forms, scroll and media state are lost on restart.

Rust reads public releases from `BadTonho/SparkBrowser`, validates the tag, manifest and installer URL, and uses the official Tauri updater. Packages need a valid signature bound to the announced version. With automatic updates off, installer bytes remain in memory until installation, channel change or exit; restarting requires downloading again. With them on, the verified installer is also kept on disk as described above. Only the local `main` WebView can invoke application update commands and receive update-state events. Direct updater-plugin IPC permissions are not granted.

See [local signed releases](../contributing/releases.md). Versions without this updater need one initial manual installation.
