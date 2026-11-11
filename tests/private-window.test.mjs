import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';

const read = path => readFileSync(path, 'utf8');
const source = read('src-tauri/src/private_session.rs');

test('private windows run as separate processes with a disposable root', () => {
  assert.match(source, /pub const SESSIONS_DIR: &str = "private-sessions";/);
  assert.match(source, /fn checked_root/);
  assert.match(read('src-tauri/src/lib.rs'), /private_session::configure\(&mut context, role\.launch\(\)\)/);
});

test('backup sync secrets are never copied into a private window', () => {
  assert.match(source, /"backup_sync_secret_v1"/);
  assert.match(source, /"backup_sync_credential_v2"/);
  assert.match(source, /substr\(key, 1, \{\}\) <> '\{EXCLUDED_PREFERENCE_PREFIX\}'/);
});

test('the open command is registered in the ACL, the handler and the shell capability', () => {
  assert.match(read('src-tauri/src/commands/manifest.rs'), /"open_private_window",/);
  assert.match(read('src-tauri/src/lib.rs'), /^\s+open_private_window,$/m);
  assert.match(read('src-tauri/capabilities/default.json'), /"allow-open-private-window"/);
});

test('closing a private window schedules removal of its session folder', () => {
  assert.match(read('src-tauri/src/shutdown/mod.rs'), /private_session::schedule_cleanup\(&app\);\s+app\.exit\(0\);/);
});

test('the shortcut opens the private window and is accepted by backups', () => {
  assert.match(read('src/lib/stores/shortcutsStore.ts'), /id: "private-window"/);
  assert.match(read('src-tauri/src/backup/validation.rs'), /"private-window",/);
  assert.match(read('src/App.svelte'), /case "private-window"/);
});
