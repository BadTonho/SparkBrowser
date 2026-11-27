import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('Data & Backup is split into sync, automatic backups and backup files, in that order', () => {
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  const order = ['backup-sync-group-heading', 'backup-local-heading', 'backup-files-heading'].map(id => page.indexOf(`id="${id}"`));
  assert.ok(order.every(index => index > 0), 'every category has its title');
  assert.deepEqual([...order].sort((a, b) => a - b), order);
  // Import and export feedback sits in the Backup files category, the last category.
  for (const piece of ['<BackupUnlockPrompt />', '<BackupPreviewPanel />', 'data-testid="backup-result"']) {
    const at = page.indexOf(piece);
    assert.ok(at > order[2], piece);
  }
  assert.doesNotMatch(read('src/lib/components/settings/BackupLocalPanel.svelte'), /<h3/, 'the category title is not repeated in the panel');
});

test('the sync card names the mode in use, and every mode has that name in both languages', () => {
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  assert.match(page, /backup\.sync_title_\$\{\$backupSyncState\.config\.mode\}/);
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/backup.json`));
    for (const mode of ['disabled', 'google_drive_web', 'folder', 'direct_webdav']) assert.ok(`backup.sync_title_${mode}` in catalog, `${mode} in ${language}`);
    for (const key of ['backup.group_files', 'backup.group_files_description', 'backup.group_sync']) assert.ok(key in catalog, `${key} in ${language}`);
  }
});

test('the open sync panel is placed by CSS, because projected settings drop inline styles', () => {
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  assert.doesNotMatch(page, /style\.setProperty|getBoundingClientRect/);
  assert.match(page, /\.sync-group \{ container-type: inline-size; \}/);
  assert.match(page, /\.sync-panel\[open\] \{ width: min\(580px, 100cqw\);/);
  // The projection really does leave inline styles out, which is why the rule above exists.
  assert.doesNotMatch(read('src/lib/services/panel-presentation.ts'), /"style"/);
});

test('the sync panel is split into titled parts, with switches and explained actions', () => {
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  for (const id of ['sync-account-title', 'sync-auto-title', 'sync-now-title']) assert.ok(page.includes(`id="${id}"`), id);
  assert.match(page, /\{#snippet switchRow\([\s\S]*?type="checkbox" role="switch"/);
  assert.doesNotMatch(page, /preference-choice/, 'no plain checkbox options are left');
  for (const testid of ['backup-sync-push', 'backup-sync-pull-preview', 'backup-sync-pull-direct', 'backup-sync-auto-save', 'backup-sync-direct-prefs']) {
    assert.ok(page.includes(`"${testid}"`), testid);
  }
  const used = new Set([...page.matchAll(/\$t\("(backup\.sync_[a-z_]+)"/g)].map(match => match[1]));
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/backup.json`));
    for (const key of used) assert.ok(key in catalog, `${key} in ${language}`);
    assert.doesNotMatch(JSON.stringify(catalog), /Auto-Merge|Choose \/ Switch|Escolher \/ Trocar/);
  }
});
