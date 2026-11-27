# Sync with Nextcloud

[Português](../../pt-BR/features/nextcloud-sync.md)

**Settings → Data & Backup → Sync → Nextcloud** syncs with your own Nextcloud server (or one from a provider). You only type the server address: there is no WebDAV address to build, no password to type into Spark and no app to register.

## Signing in

1. Choose **Nextcloud** as the sync mode and type the server address, such as `cloud.example.com`. `https://` is added when missing, and a pasted address of a Nextcloud page (`.../index.php/apps/files/...`, `.../login`) is cut back to the server. Plain `http://` is only accepted for `localhost`, like WebDAV.
2. Select **Sign in to Nextcloud**. Settings close and the server's login page opens in a tab.
3. Sign in there and allow Spark. Spark notices within about 2 seconds, closes that tab and opens Settings again on Data & Backup, connected.

While Spark waits, the sync panel says so and offers **Cancel sign-in**. Closing the login tab also cancels it. A sign-in left unfinished expires after 20 minutes, like Nextcloud's own.

Once connected, the panel shows the account and server in use. **Sign in again** gets a new app password (for example after removing Spark in Nextcloud), and the **Automatic** options apply at once.

## Where the file goes

Spark creates a **Spark** folder in your files (if it does not exist yet) and keeps `browser.spark-backup.json` there. From then on the sync is the same as [WebDAV](backup.md): the same automatic sends, receive modes, **What syncs**, **Download when Spark opens** and **Earlier versions** (the newest 5 copies in `Spark/versions`). The file is encrypted the same way when backup encryption is on.

## Security

- Spark uses Nextcloud's login flow for apps (Login Flow v2). Your Nextcloud password is typed only on your server's page; Spark receives an **app password** made for it alone.
- The app password is kept in the system credential store (Windows Credential Manager or the Linux Secret Service), like a remembered WebDAV password, and never reaches the interface. The poll token of the login also stays in Rust.
- Spark only follows a login page and poll address on the same server (same scheme, host and port) as the one you typed, and never follows redirects.
- Nextcloud lists Spark as **Spark Browser (sync)** in **Settings → Security → Devices & sessions**, where you can remove it at any time. Spark then gets an authentication error until you sign in again.
- Changing to another sync mode forgets the app password on this computer. It does not revoke it on the server: remove it there if you no longer use it.

## How it works

- Rust: `src-tauri/src/backup/nextcloud.rs` cleans up the server address, runs the login (`POST /index.php/login/v2`, then polls the returned endpoint with its token until it answers with the login name and app password) and creates the folder (`MKCOL`). The commands are in `src-tauri/src/commands/backup_nextcloud.rs`: `nextcloud_login_start`, `nextcloud_login_poll` and `nextcloud_login_cancel`.
- When the person allows Spark, the poll saves a WebDAV target (`direct_webdav`) pointed at `<server>/remote.php/dav/files/<login name>/Spark/`, with the app password remembered and `nextcloud_server` set. That field is what makes the interface show it as Nextcloud. `save_config` checks it is a valid server on the same host as the WebDAV address.
- Interface: `src/lib/stores/nextcloudLoginStore.ts` opens the login tab, polls every 2 seconds and gives up when the tab is closed. The sync panel is in `BackupSettings.svelte`.
- Failures are written to the error log as Warnings with the step that failed (start or sign-in), and the panel shows the message.

Tests: `src-tauri/src/backup/nextcloud.rs` (address clean-up, folder address, origin check) and `tests/backup-nextcloud.test.mjs` (commands registered end to end, messages in both languages, app password kept out of the interface).
