import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8').replace(/\r\n/g, '\n');
function load(path) {
  const module = { exports: {} };
  const output = ts.transpileModule(read(path), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
  runInNewContext(output, { module, exports: module.exports });
  return module.exports;
}
const { summarizeSyncRing } = load('src/lib/services/backup/sync-ring.ts');
const base = { mode: 'google_drive_web', autoSave: true, running: 0, finished: false, problem: null, waitingForAccount: false };
const ring = changes => summarizeSyncRing({ ...base, ...changes });

test('the sync ring follows the download button: syncing, done, paused and error', () => {
  assert.equal(ring({}), 'idle');
  assert.equal(ring({ running: 1 }), 'syncing');
  assert.equal(ring({ running: 1, problem: 'old failure' }), 'syncing', 'a running sync wins over an older problem');
  assert.equal(ring({ problem: 'backup.sync_failed' }), 'error');
  assert.equal(ring({ finished: true }), 'done');
  assert.equal(ring({ autoSave: false }), 'paused', 'automatic sending turned off');
  assert.equal(ring({ problem: 'link needed', waitingForAccount: true }), 'paused', 'waiting for a Google account is not an error');
  assert.equal(ring({ finished: true, autoSave: false }), 'done', 'a manual sync still shows that it finished');
  assert.equal(ring({ mode: 'disabled', running: 1, problem: 'x' }), 'idle', 'no ring while sync is off');
});

test('the ring is drawn with classes only and every sync is tracked', () => {
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  for (const state of ['syncing', 'done', 'paused', 'error']) assert.match(page, new RegExp(`class:ring-${state}=`), state);
  assert.match(page, /\.sync-slot::before \{[^}]*conic-gradient/);
  assert.match(read('src/lib/components/settings/settings.css'), /@property --sync-fill/);
  const store = read('src/lib/stores/backupStore.ts');
  for (const name of ['syncPushNow', 'syncPullPreview', 'syncPullDirect']) {
    assert.match(store, new RegExp(`export const ${name} = [^\\n]*\\n  action\\(\\(\\) => trackSync\\(`), name);
  }
  assert.match(store, /async function runBackgroundAutoSync[\s\S]*?await trackSync\(/, 'the automatic sync lights the ring too');
  // The Drive pause is only changed by its setter, which also tells the interface.
  assert.equal(store.match(/gdriveLoginRequiredPause = /g).length, 2, 'the declaration and the setter');
  assert.match(store, /function setDrivePause\(paused: boolean\): void \{\s*gdriveLoginRequiredPause = paused;\s*driveWaitingForAccount\.set\(paused\);/);
});
