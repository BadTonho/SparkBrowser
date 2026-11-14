# Bookmark export for other browsers

Settings → Data & Backup has a **Bookmarks** card with two buttons that write every bookmark to a file other browsers can read. This is separate from the `.veloce-backup.json` backup, which only this app restores.

## Formats

- **HTML** (`.html`, `.htm`) is the Netscape bookmark format. Chrome, Edge, Firefox, Safari, Brave, Vivaldi and most bookmark managers import it (for example Chrome → Bookmarks → Import bookmarks, Firefox → Bookmarks → Manage → Import).
- **CSV** (`.csv`) has the columns `folder,title,url`, one bookmark per row, for spreadsheets and scripts. It is UTF-8 with a byte order mark so Excel shows accented titles correctly, with CRLF line endings and RFC 4180 quoting.

Folders are exported as one level, in folder order, followed by bookmarks that have no folder. Empty folders are left out. A bookmark whose folder no longer exists is exported as a loose one. Icons and creation dates are not exported.

## Safety

- Only plain `http` and `https` addresses are exported. Anything else (`javascript:`, `data:` and similar) is skipped and counted in the notice shown after exporting, so a hostile bookmark never lands in another browser's toolbar.
- HTML titles, folder names and addresses are escaped, and control characters are dropped.
- CSV cells that start with `=`, `+`, `-`, `@`, a tab or a carriage return get a leading apostrophe so a spreadsheet will not run a page title as a formula.
- The file name you confirm in the save dialog is never changed afterwards. A name that does not match the format (`.html`/`.htm` or `.csv`) is refused instead of silently renamed. The file is written to a temporary file and moved into place, so a failure keeps any previous file.
- With no bookmarks to export the command reports that instead of writing an empty file. Cancelling the dialog does nothing.

Bookmarks are not secret data, but the file contains your reading list in plain text. Nothing leaves the computer.

## Not included yet

- Importing bookmarks from another browser's HTML file.
- Exporting saved sessions, notes or tabs in these formats.

## Implementation

- Formatting is pure code in `src-tauri/src/backup/bookmark_export.rs` (tests included).
- The `export_bookmarks` command in `src-tauri/src/commands/bookmark_export.rs` reads the bookmarks, opens the native save dialog from Rust and writes atomically. The Tauri dialog plugin's JavaScript commands stay ungranted.
- The interface is `BookmarkExportCard.svelte`, backed by `exportBookmarksAs` in `backupStore.ts`.
