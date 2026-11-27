import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8').replace(/\r\n/g, '\n');

test('earlier versions are listed and opened in the review panel, never applied directly', () => {
  const store = read('src/lib/stores/backupStore.ts');
  assert.match(store, /export const reviewSyncVersion = \(id: string\) =>[\s\S]*?source = res\.source_token;\n      await createPreview\(\);/);
  assert.doesNotMatch(store.slice(store.indexOf('export const reviewSyncVersion')), /^.*applyImportedResult[\s\S]*?export const syncPullPreview/m);
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  assert.match(page, /data-testid="backup-sync-versions-load"/);
  assert.match(page, /data-testid="backup-sync-version-review"/);
});

test('the Drive bridge checks revision ids and every command is registered', () => {
  const bridge = read('src-tauri/src/backup/gdrive_bridge.js');
  assert.match(bridge, /const REVISION_ID = \/\^\[A-Za-z0-9_-\]\{1,200\}\$\/;/);
  assert.match(bridge, /window\.__sparkGDriveVersions = async \(token, requestId\) => \{\n    if \(token !== BRIDGE_TOKEN\) return;/);
  assert.match(bridge, /window\.__sparkGDriveVersion = async \(token, requestId, revisionId\) => \{\n    if \(token !== BRIDGE_TOKEN\) return;/);
  assert.match(read('src-tauri/src/backup/gdrive_webview.rs'), /if !super::sync_versions::valid_revision_id\(revision_id\) \{/);
  for (const command of ['list_backup_sync_versions', 'review_backup_sync_version']) {
    assert.match(read('src-tauri/src/commands/manifest.rs'), new RegExp(`"${command}"`));
    assert.match(read('src-tauri/capabilities/default.json'), new RegExp(`"allow-${command.replaceAll('_', '-')}"`));
    assert.match(read('src-tauri/src/lib.rs'), new RegExp(`${command},\n`));
    assert.match(read('src/lib/services/bridge/backup.ts'), new RegExp(`"${command}"`));
  }
  const errors = read('src/lib/i18n/errors.ts');
  for (const code of ['sync_versions_failed', 'sync_version_not_found']) {
    assert.match(errors, new RegExp(`"backup\.${code}": "backup\.error_${code}"`));
    for (const language of ['en', 'pt-BR']) assert.ok(`backup.error_${code}` in JSON.parse(read(`src/lib/i18n/catalogs/${language}/backup.json`)), `${code} in ${language}`);
  }
});

test('a send keeps the replaced file only when the content changed, and never fails because of it', () => {
  const sync = read('src-tauri/src/backup/sync.rs');
  assert.match(sync, /let keep_version = previous_fp\.as_deref\(\) != Some\(current_fp\.as_str\(\)\);/);
  assert.equal(sync.match(/report_version_failure\(/g).length, 3, 'definition plus folder and WebDAV');
});
