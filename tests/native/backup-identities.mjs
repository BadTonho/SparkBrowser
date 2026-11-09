import assert from 'node:assert/strict';
import { backupPreferences } from './backup.mjs';

export async function checkBackupIdentities({ invoke, fixture, restart }) {
  const account = (await invoke('create_account_container', { name: 'Backup fixture account' })).find(item => item.name === 'Backup fixture account');
  const regular = (await invoke('create_container_tab', { id: account.id })).tabs.find(tab => tab.is_active);
  const ephemeral = (await invoke('create_ephemeral_tab', { url: fixture.origin + '/page/backup-private' })).tabs.find(tab => tab.is_active);
  const before = await invoke('get_tabs');
  const prefs = backupPreferences();
  const source = { format: 'veloce-backup', version: 4, created_at: 1, preferences: prefs,
    data: { bookmarks: await invoke('get_bookmark_data'), workspaces: await invoke('get_workspaces'),
      tabs: before.tabs.filter(tab => !tab.is_ephemeral).map(({ id, title, url, workspace_id, is_pinned }) => ({ id, title, url, workspace_id, is_pinned, is_active: false })),
      split_layouts: [], sessions: [], notes: [], site_search: [] } };
  source.data.notes.push({ url: fixture.origin + '/backup-identity-note', title: 'Imported fixture', text: 'Keep local tabs', updated_at: 1 });
  const preview = await invoke('preview_backup_import', { document: source, currentPreferences: prefs });
  const result = await invoke('commit_backup_import', { token: preview.token, applyPreferences: false, currentPreferences: prefs });
  assert.deepEqual(result.snapshot.tabs.map(tab => tab.id), before.tabs.map(tab => tab.id));
  assert.equal(result.snapshot.tabs.find(tab => tab.id === regular.id).container_id, account.id);
  assert.ok(result.snapshot.tabs.find(tab => tab.id === ephemeral.id)?.is_active, 'import preserves the active disposable tab');
  await restart({ crash: true });
  const restored = (await invoke('get_tabs')).tabs;
  assert.equal(restored.find(tab => tab.id === regular.id)?.container_id, account.id);
  assert.ok(!restored.some(tab => tab.id === ephemeral.id));
}
