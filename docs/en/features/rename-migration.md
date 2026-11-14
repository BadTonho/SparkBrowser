# Upgrading from the Veloce name

The app was called Veloce before it became Spark. Everything that was saved under the old name keeps working: the first launch of a Spark build moves and converts it, once, without asking.

## What moves on its own

| Data | What happens |
| --- | --- |
| Profile folders (`org.veloce.browser` → `org.spark.browser`) | Renamed before any webview opens, so bookmarks, settings and the WebView profile (cookies, site storage) come along. Windows moves both Roaming and Local app data; Linux moves data, config and cache. Nothing moves if the new folder already exists. |
| Database file (`veloce_browser.db` → `spark_browser.db`) | Renamed with its `-wal` and `-shm` files. |
| Saved interface settings (`veloce_*` keys → `spark_*`) | Copied on startup, before any setting is read; the old key is removed. A value already stored under the new name is never overwritten. |
| Saved tabs, sessions and recently closed tabs holding `veloce://newtab` | Rewritten to `spark://newtab` once. The old address is still recognised if it shows up in an imported file. |
| WebDAV and backup-encryption passwords in the system credential store | Copied from the old entry names to the new ones, then the old entries are removed. If the keyring is locked, the next start tries again. |
| Search index triggers in existing databases | They call `veloce_search_fold`, so the app keeps registering that function next to `spark_search_fold`. |

## Backups and sync files

- Backup files keep opening: `.veloce-backup.json` files, documents tagged `veloce-backup` and encrypted files tagged `veloce-backup-encrypted` are all read. New files use `.spark-backup.json`, `spark-backup` and `spark-backup-encrypted`.
- The encryption itself is unchanged. The associated data bound into every encrypted file keeps its old text on purpose, because changing it would make existing encrypted backups impossible to decrypt.
- Local history lists both `…veloce-backup.json` and `…spark-backup.json` entries; new snapshots use the new name.
- Sync targets now write `browser.spark-backup.json`. Pulls look for the new name first and fall back to `browser.veloce-backup.json`: in a synced folder, on WebDAV (when the endpoint is a folder) and on Google Drive. On Drive the lookup matches either name, so an existing file is found and updated in place instead of a second copy appearing. The old Drive folder name `Veloce Browser Backup` is still searched too.

## What does not carry over

- **New tab page storage.** The new tab page is served from its own address (`spark-internal://…`), which is a different origin from before. Its wallpaper and clock settings are stored per origin and start from defaults.
- **Unfinished downloads.** Partial files of resumable downloads moved from `.veloce-partials` to `.spark-partials` in the download folder. A download that was still in progress cannot resume; finished downloads are not affected, and the old hidden folder can be deleted.
- Anything created by a build that never ran, such as test profiles: native test builds use their own `org.spark.browser.native-tests` identifier.

## For developers

- The release signing key may stay in `~/.veloce-signing`: `release:build` and `release:setup-signing` use `~/.spark-signing` when it exists and otherwise the old folder, and honour `SPARK_SIGNING_KEY_PATH` or the old `VELOCE_SIGNING_KEY_PATH`. Moving the key is optional.
- Test environment variables are now `SPARK_NATIVE_*`; crate `spark-browser`, library `spark_browser_lib`.
- Code: `src-tauri/src/legacy.rs` (profile and database move), `backup/credentials.rs` (secrets), `database/connection.rs` (address rewrite), `src/lib/services/storage.js` (settings keys). Tests: `legacy_tests.rs`, `tests/rename-compat.test.mjs` and the `rename_tests` module in `connection.rs`.
- The pre-rename names can be removed after a few releases, when no profile older than the rename is expected any more; each piece above is isolated for that.
