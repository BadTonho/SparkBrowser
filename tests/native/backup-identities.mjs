import assert from 'node:assert/strict';
import { backupPreferences } from './backup.mjs';

export async function checkBackupIdentities({ invoke, fixture, restart, until, withWebview, script }) {
  const account = (await invoke('create_account_container', { name: 'Backup fixture account' })).find(item => item.name === 'Backup fixture account');
  const regular = (await invoke('create_container_tab', { id: account.id })).tabs.find(tab => tab.is_active);
  await invoke('navigate_to', { input: fixture.origin + '/page/backup-account' });
  await until(() => fixture.reports.has('backup-account'), 'account document ready');
  await withWebview('/page/backup-account', () => script(`const input=document.querySelector('#draft'); input.value='preserved import draft'; input.dispatchEvent(new Event('input',{bubbles:true}));`));
  const companion = (await invoke('create_tab', { url: fixture.origin + '/page/backup-companion' })).tabs.find(tab => tab.is_active);
  const ephemeral = (await invoke('create_ephemeral_tab', { url: fixture.origin + '/page/backup-private' })).tabs.find(tab => tab.is_active);
  await invoke('set_split_view', { leftTabId: regular.id, rightTabId: companion.id });
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
  assert.ok(result.snapshot.tabs.some(tab => tab.id === ephemeral.id), 'import preserves the live disposable tab');
  assert.equal(result.snapshot.tabs.find(tab => tab.is_active)?.id, before.tabs.find(tab => tab.is_active)?.id);
  assert.deepEqual(result.snapshot.split_layout, before.split_layout);
  await withWebview('/page/backup-account', async () => assert.equal(await script(`return document.querySelector('#draft').value`), 'preserved import draft'));
  assert.equal(fixture.loads.get('backup-account'), 1, 'import must reuse the account document');
  await restart({ crash: true });
  const packet = await invoke('get_tabs');
  const restored = packet.tabs;
  assert.equal(restored.find(tab => tab.id === regular.id)?.container_id, account.id);
  assert.ok(!restored.some(tab => tab.id === ephemeral.id));
  assert.equal(packet.split_layout?.left_tab_id, regular.id);
  assert.equal(packet.split_layout?.right_tab_id, companion.id);
}
