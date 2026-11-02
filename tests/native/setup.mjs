import assert from 'node:assert/strict';
export async function prepareTabs({ invoke, until, fixture }) {
  await invoke('set_resident_limit', { limit: 8 });
  const fixtureWorkspace = 'native-fixture-' + Date.now();
  await invoke('create_workspace', { id: fixtureWorkspace, name: fixtureWorkspace, icon: 'T', accentPrimary: '#6e8efb', accentSecondary: '#8b949e' });
  const a = (await invoke('create_tab', { url: fixture.origin + '/page/a', searchEngine: 'google' })).tabs.find(tab => tab.is_active);
  await invoke('move_tab_to_workspace', { tabId: a.id, workspaceId: fixtureWorkspace });
  a.workspace_id = fixtureWorkspace;
  await until(() => fixture.reports.get('a')?.value === 'artificial-test-draft', 'fixture form report');
  await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id)?.activity.dirty_form, 'edited form protection');
  fixture.commands.set('a', 'spa');
  await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id)?.url.endsWith('/spa'), 'SPA address update');
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/b', searchEngine: 'google' })).tabs.find(tab => tab.is_active);
  await until(() => fixture.reports.has('b'), 'second page');
  for (let i = 0; i < 8; i++) await invoke('switch_tab', { tabId: i % 2 ? b.id : a.id });
  await invoke('switch_tab', { tabId: a.id });
  assert.equal(fixture.loads.get('a'), 1, 'switching resident tabs must not reload forms');
  assert.equal(fixture.reports.get('a').value, 'artificial-test-draft');
  assert.ok(fixture.reports.get('a').scroll >= 500, 'switching tabs must preserve scroll position');
  await invoke('set_tab_pinned', { tabId: b.id, pinned: true });
  await invoke('set_resident_limit', { limit: 1 });
  const protectedTabs = (await invoke('get_tabs')).tabs;
  assert.equal(protectedTabs.find(tab => tab.id === b.id).is_hibernated, false);
  assert.equal(protectedTabs.find(tab => tab.id === a.id).is_hibernated, false);
  await invoke('set_resident_limit', { limit: 8 });

  return { a, b, fixtureWorkspace };
}
