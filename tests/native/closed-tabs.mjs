import assert from 'node:assert/strict';
import { backupPreferences } from './backup.mjs';

export async function checkClosedTabs({ invoke, until, fixture, restart, script }) {
  await invoke('set_resident_limit', { limit: 8 });
  const ws = 'closed-tabs-fixture';
  await invoke('create_workspace', { id: ws, name: 'Closed tabs fixture', icon: 'T', accentPrimary: '#123456', accentSecondary: '#654321' });
  await invoke('switch_workspace', { workspaceId: ws });
  const create = async name => {
    const tab = (await invoke('create_tab', { url: fixture.origin + `/page/${name}` })).tabs.find(tab => tab.is_active);
    await until(async () => !(await invoke('get_tabs')).tabs.find(item => item.id === tab.id)?.navigation.loading, 'fixture loaded');
    return tab;
  };
  const first = await create('first');
  const second = await create('second');
  await invoke('set_tab_pinned', { tabId: first.id, pinned: true });
  const original = (await invoke('get_tabs')).tabs.filter(tab => tab.workspace_id === ws);
  await invoke('close_tab', { tabId: first.id });
  let rows = await invoke('get_closed_tabs');
  assert.equal(rows.length, 1);
  assert.equal(rows[0].position, original.findIndex(tab => tab.id === first.id));
  assert.equal(rows[0].is_pinned, true);
  const recordId = rows[0].id;
  await restart();
  assert.equal((await invoke('get_closed_tabs'))[0].id, recordId);
  const reopened = (await invoke('reopen_closed_tab', { id: recordId })).snapshot.tabs;
  const restored = reopened.find(tab => tab.is_active);
  assert.notEqual(restored.id, first.id, 'restoration after restart must allocate a new ID');
  assert.equal(restored.is_pinned, true);
  assert.equal(restored.workspace_id, ws);
  assert.equal(reopened.filter(tab => tab.workspace_id === ws).findIndex(tab => tab.id === restored.id), rows[0].position);
  assert.equal((await invoke('get_closed_tabs')).length, 0);

  await invoke('switch_tab', { tabId: second.id });
  await invoke('set_tab_pinned', { tabId: restored.id, pinned: false });
  await until(async () => !(await invoke('get_tabs')).tabs.find(tab => tab.id === restored.id).navigation.loading, 'restored fixture loaded');
  await invoke('hibernate_tab', { tabId: restored.id });
  assert.equal((await invoke('get_closed_tabs')).length, 0, 'hibernation is not a closure');
  const bulk = await invoke('close_tabs_reviewed', { tabIds: [restored.id, second.id] });
  assert.deepEqual(bulk.closed, [restored.id]);
  assert.equal(bulk.preserved[0].id, second.id);
  await script(`document.dispatchEvent(new KeyboardEvent('keydown', { key: 'T', ctrlKey: true, shiftKey: true, bubbles: true, cancelable: true }));`);
  await until(async () => !(await invoke('get_closed_tabs')).length, 'Ctrl Shift T reopens the latest closure');
  const shortcutTab = (await invoke('get_tabs')).tabs.find(tab => tab.is_active);
  assert.equal(shortcutTab.url, restored.url);
  await invoke('close_tab', { tabId: shortcutTab.id });
  await invoke('delete_workspace', { workspaceId: ws, action: 'close', targetWorkspaceId: null });
  assert.equal((await invoke('get_closed_tabs')).length, 1, 'workspace deletion must not record its tabs');
  const fallback = await invoke('reopen_closed_tab', { id: null });
  assert.equal(fallback.workspace_missing, true);
  assert.equal(fallback.snapshot.tabs.find(tab => tab.is_active).workspace_id, 'ws-personal');

  // Import a hibernated blocked page without ever contacting an external site.
  const packet = await invoke('get_tabs');
  const prefs = backupPreferences();
  const document = { format: 'veloce-backup', version: 1, created_at: 1, preferences: prefs, data: {
    workspaces: await invoke('get_workspaces'), bookmarks: { folders: [], bookmarks: [] }, sessions: [], notes: [], site_search: [], split_layouts: [],
    tabs: [...packet.tabs.map(({ id, title, url, workspace_id, is_active, is_pinned }) => ({ id, title, url, workspace_id, is_active, is_pinned })),
      { id: 'blocked-foreign', title: 'Blocked fixture', url: 'https://doubleclick.net/fixture', workspace_id: 'ws-personal', is_active: false, is_pinned: false }],
  } };
  const preview = await invoke('preview_backup_import', { document, currentPreferences: prefs });
  const imported = await invoke('commit_backup_import', { token: preview.token, currentPreferences: prefs, applyPreferences: false });
  const blocked = imported.snapshot.tabs.find(tab => tab.url === 'https://doubleclick.net/fixture');
  assert.equal(blocked.is_hibernated, true);
  await invoke('close_tab', { tabId: blocked.id });
  const blockedRecord = (await invoke('get_closed_tabs'))[0];
  await assert.rejects(invoke('reopen_closed_tab', { id: blockedRecord.id }), /closed_tabs.blocked/);
  assert.equal((await invoke('get_closed_tabs'))[0].id, blockedRecord.id, 'shield denial keeps the record');
  await script(`document.querySelector('[data-testid="open-tools"]').click();`);
  await until(() => script(`return Boolean(document.querySelector('[data-testid="tools-closed"]'))`), 'closed tabs section');
  await script(`document.querySelector('[data-testid="tools-closed"]').click();`);
  await until(() => script(`return document.querySelectorAll('[data-testid="closed-tab-record"]').length === 1`), 'closed tabs list');
  await script(`document.querySelector('[data-testid="close-tools"]').click();`);
  await invoke('clear_browsing_history');
  assert.equal((await invoke('get_closed_tabs')).length, 0);
}
