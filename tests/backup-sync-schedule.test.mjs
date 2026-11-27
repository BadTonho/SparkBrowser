import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8').replace(/\r\n/g, '\n');
function load(path) {
  const module = { exports: {} };
  const output = ts.transpileModule(read(path), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
  runInNewContext(output, { module, exports: module.exports, Date, Math });
  return module.exports;
}
const { tickShouldSync, syncPaused, followsEachChange, pauseUntil } = load('src/lib/services/backup/sync-schedule.ts');
const now = 1_800_000_000_000;
const options = (interval, paused_until = null) => ({ interval, paused_until });
const tick = (mode, opts, pending = true, sinceLastMs = 60_000) => tickShouldSync({ mode, options: opts, pending, sinceLastMs, nowMs: now });

test('the automatic frequency keeps the original behavior', () => {
  assert.equal(tick('google_drive_web', options('auto')), true);
  assert.equal(tick('direct_webdav', options('auto')), true);
  assert.equal(tick('folder', options('auto')), false, 'a folder follows each change instead');
  assert.equal(tick('google_drive_web', options('auto'), false), false, 'nothing new, nothing sent');
  assert.equal(followsEachChange('auto'), true);
  assert.equal(followsEachChange('five_minutes'), false);
});

test('longer frequencies wait their time, and "on exit" never sends on the tick', () => {
  assert.equal(tick('folder', options('five_minutes'), true, 4 * 60_000), false);
  assert.equal(tick('folder', options('five_minutes'), true, 5 * 60_000), true);
  assert.equal(tick('google_drive_web', options('fifteen_minutes'), true, 14 * 60_000), false);
  assert.equal(tick('google_drive_web', options('fifteen_minutes'), true, 15 * 60_000), true);
  assert.equal(tick('direct_webdav', options('on_exit'), true, 99 * 60_000), false);
  assert.equal(tick('disabled', options('auto')), false);
});

test('a pause stops the tick until it ends', () => {
  const later = now / 1000 + 60;
  assert.equal(syncPaused(options('auto', later), now), true);
  assert.equal(tick('google_drive_web', options('auto', later)), false);
  assert.equal(syncPaused(options('auto', now / 1000 - 1), now), false, 'an old pause is over');
  const date = new Date(2026, 10, 27, 15, 30);
  assert.equal(pauseUntil('hour', date), Math.floor(date.getTime() / 1000) + 3600);
  assert.equal(pauseUntil('tomorrow', date), Math.floor(new Date(2026, 10, 28, 0, 0).getTime() / 1000));
});

test('the store respects the pause and the frequency, and warns once after repeated failures', () => {
  const store = read('src/lib/stores/backupStore.ts');
  assert.match(store, /if \(get\(syncPausedNow\)\) return;\n  lastAutoSyncAt = Date\.now\(\);/, 'background sync skips while paused');
  assert.match(store, /!cfg\.auto_save \|\| get\(syncPausedNow\)\) return;/, 'closing does not send while paused');
  assert.match(store, /followsEachChange\(options\.interval\)/);
  assert.match(store, /tickShouldSync\(\{/);
  assert.match(store, /if \(autoSyncFailures !== FAILURE_NOTICE_AFTER\) return;/, 'the notice shows once per streak');
  assert.match(store, /autoSyncFailures = 0;/, 'a success resets the streak');
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/backup.json`));
    for (const key of ['backup.sync_failure_notice', 'backup.sync_paused_until', 'backup.sync_last_from', 'backup.sync_interval_on_exit']) assert.ok(key in catalog, `${key} in ${language}`);
  }
});

test('the backend registers the options command and writes the device name (format 7)', () => {
  assert.match(read('src-tauri/src/commands/manifest.rs'), /"set_backup_sync_options"/);
  assert.match(read('src-tauri/capabilities/default.json'), /"allow-set-backup-sync-options"/);
  assert.match(read('src-tauri/src/lib.rs'), /set_backup_sync_options,\n/);
  assert.match(read('src/lib/services/bridge/backup.ts'), /"set_backup_sync_options"/);
  assert.match(read('src-tauri/src/backup/model.rs'), /pub const CURRENT_VERSION: u32 = 7;/);
  assert.match(read('src-tauri/src/backup/service.rs'), /device: Some\(super::device::name\(\)\)/);
});
