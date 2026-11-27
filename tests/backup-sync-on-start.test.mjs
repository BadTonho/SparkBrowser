import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8').replace(/\r\n/g, '\n');

test('"Download when Spark opens" reads the cloud before the first automatic send', () => {
  const store = read('src/lib/stores/backupStore.ts');
  assert.match(store, /if \(state\.config\.mode !== "disabled" && state\.options\.pull_on_start\) \{[\s\S]*?runSyncOnStart\(\)\.finally\(\(\) => \{ if \(scope\.active\(\)\) firstSend\(\); \}\)/);
  assert.match(store, /async function runSyncOnStart\(\)[\s\S]*?get\(syncPausedNow\)\) return;/, 'a pause also stops the download on start');
  assert.match(store, /res\.kind === "applied"[\s\S]*?applyImportedResult\(res\.result\)/);
  assert.match(store, /res\.kind === "review"[\s\S]*?source = res\.source_token;[\s\S]*?createPreview\(\)/, 'a conflict opens the review instead of applying');
});

test('the backend decides from fingerprints and the command is registered end to end', () => {
  const sync = read('src-tauri/src/backup/sync.rs');
  assert.match(sync, /pub fn decide_on_start\(remote: &str, local: &str, last_synced: Option<&str>\) -> StartDecision/);
  assert.match(sync, /Err\(error\) if error == "backup\.sync_file_not_found"/, 'an empty cloud is not a failure');
  assert.match(read('src-tauri/src/backup/sync_options.rs'), /pub pull_on_start: bool,/);
  assert.match(read('src-tauri/src/commands/manifest.rs'), /"sync_backup_on_start"/);
  assert.match(read('src-tauri/capabilities/default.json'), /"allow-sync-backup-on-start"/);
  assert.match(read('src-tauri/src/lib.rs'), /sync_backup_on_start,\n/);
  assert.match(read('src/lib/services/bridge/backup.ts'), /"sync_backup_on_start"/);
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  assert.equal(page.match(/\{@render pullOnStartRow\(\)\}/g).length, 3, 'the switch is offered in every mode');
});
