import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8').replace(/\r\n/g, '\n');

test('"What syncs" offers every group and keeps at least one on', () => {
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  assert.match(page, /const SYNC_GROUPS: RestoreGroup\[\] = \["workspaces", "bookmarks", "sessions", "notes", "site_search", "settings", "shortcuts"\];/);
  assert.match(page, /disabled=\{\$backupBusy \|\| \(on && \$backupSyncState\.options\.groups\.length === 1\)\}/, 'the last group cannot be turned off');
  assert.match(page, /if \(next\.length\) void saveSyncOptions\(\{ groups: next \}\);/);
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/backup.json`));
    for (const key of ['backup.sync_section_groups', 'backup.sync_groups_hint', 'backup.group_site_search', 'backup.group_shortcuts']) assert.ok(key in catalog, `${key} in ${language}`);
  }
});

test('every sync path filters by the groups of this computer', () => {
  const sync = read('src-tauri/src/backup/sync.rs');
  assert.doesNotMatch(sync, /fingerprint_hex\((&?document|&remote|&local|&synced_doc)\)/, 'no sync path fingerprints unfiltered data');
  assert.equal(sync.match(/sync_restore_options\(&groups\)/g).length, 3, 'the direct pull, the start-up apply and the version review only accept the synced groups');
  assert.match(sync, /super::sync_filter::filter\(&mut document, &groups\);/, 'only the synced groups leave this computer');
  assert.match(read('src-tauri/src/backup/sync_options.rs'), /groups: RestoreGroup::ALL\.to_vec\(\),/, 'everything syncs by default');
});
