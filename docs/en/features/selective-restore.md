# Selective restore and replace mode

Every restore (a chosen file, a history entry or **Review before applying** in the sync panel) opens the review panel before anything is written. The panel shows when the backup was created and its file format version, then lets you decide what is restored and how.

## What to restore

| Option | Restores |
| --- | --- |
| Workspaces, tabs and splits | Workspaces, open tabs and split layouts (they depend on each other) |
| Bookmarks and folders | Bookmarks and their folders |
| Saved sessions | Named sessions |
| Page notes | Plain-text notes |
| Site searches | Keyword searches |
| Settings and appearance | Every preference except keyboard shortcuts |
| Keyboard shortcuts | The shortcut map |

All data options start checked. The two preference options start unchecked, so a restore never changes local settings unless you ask. Changing any option recalculates the preview, so the counts always describe exactly what will be written. At least one option must be selected.

Preference groups mix sources: choosing only shortcuts keeps every current setting, and choosing only settings keeps the current shortcuts. A pending preference task from an earlier restore must finish first.

## How to restore

- **Merge with current data** (default) keeps current values on conflicts and only adds what is missing, as described in [Local backup and reviewed import](backup.md).
- **Replace current data** deletes the selected current items and writes the backup's content instead. The table gains a **To remove** column and a warning shows how many current items will be permanently deleted. The confirm button is renamed **Replace and import**.

Replace applies to bookmarks, saved sessions, notes and site searches. Workspaces, tabs and splits are always merged, because replacing them would close live tabs and webviews.

Replacement runs in the same SQLite transaction as the rest of the import: if any write fails, nothing is deleted. The preview is still bound to current data, so a stale preview must be refreshed before confirming.

## Safety snapshot and undo

Before a replace that would delete anything, the app saves the current data to the local backup history with the reason **Before replace** (encrypted when encryption is on). If that copy cannot be written, the replace does not happen. After the import, the result shows **Undo replace…**, which opens the safety copy as a replace-mode review of the same groups; confirming it restores the previous data (and takes a new safety copy, so the undo can itself be undone).

Safety copies do not count toward the history version limit or age setting; they keep their own policy (the 5 newest, for at most 30 days). They do not reset the automatic backup schedule. Merge imports delete nothing and take no safety copy.

## Not included yet

- Browsing history is not part of the backup format, so it cannot be restored selectively.
- The backup file does not record the app version, only the file format version.
- Selection applies when restoring. Exports, history snapshots and sync files always contain everything.

## Implementation

- `RestoreOptions` (`groups`, `mode`) is sent with `preview_backup_import`; the commit reuses the options stored with the preview token.
- `merge::plan_with` filters the incoming data by group, empties replaced groups in a working copy and merges into it, then reports `removed` per kind.
- `commit_backup_import` calls `local::create(..., Reason::PreRestore)` first when `BackupService::deletes_local_data` is true.
- `backup_repo::commit_with` wipes replaced groups inside the import transaction.
- The interface lives in `BackupPreviewPanel.svelte`; state is in `backupStore.ts` (`restoreOptions`, `updateRestoreOptions`).
- Rust tests: `src-tauri/src/backup/selective_tests.rs`.
