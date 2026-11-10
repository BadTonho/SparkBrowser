import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

export async function checkAccountContainers({ invoke, until, fixture, restart, directory }) {
  const accounts = await invoke('create_account_container', { name: 'Fixture A' });
  const a = accounts.find(item => item.name === 'Fixture A');
  const b = (await invoke('create_account_container', { name: 'Fixture B' })).find(item => item.name === 'Fixture B');
  assert.ok(a && b && a.id !== b.id);
  await invoke('set_workspace_isolation', { enabled: true });
  // Isolated profiles run outside the shell's WebDriver process. The loopback
  // fixtures report only synthetic values, without adding application IPC grants.
  const reported = async (id, minimumLoad = 1) => until(() => {
    const report = fixture.reports.get(id);
    return report?.pageLoad >= minimumLoad && report.account ? report.account : false;
  }, `account storage report: ${id} load ${minimumLoad}`);
  const check = async (id, expected, minimumLoad = 1) => {
    const account = await reported(id, minimumLoad);
    assert.equal(account.local, expected, `${id}: localStorage`);
    assert.equal(account.indexed, expected, `${id}: IndexedDB`);
    assert.equal(account.cookie.includes('fixture_account=alpha'), expected === 'alpha', `${id}: cookie`);
  };
  const open = async (id, page) => {
    const snapshot = await invoke('create_container_tab', { id });
    const tab = snapshot.tabs.find(item => item.is_active);
    await until(async () => {
      const current = (await invoke('get_tabs')).tabs.find(item => item.id === tab.id);
      return current?.url === 'veloce://newtab' && current.navigation.revision > 0
        && !current.navigation.loading && !current.navigation.error;
    }, 'container initial document ready');
    await invoke('navigate_to', { input: `${fixture.origin}/page/${page}` });
    await reported(page);
    return tab;
  };
  const first = await open(a.id, 'account-a');
  fixture.commands.set('account-a', 'store-account-alpha');
  await until(() => fixture.reports.get('account-a')?.account?.indexed === 'alpha', 'seed container A');
  const second = await open(a.id, 'account-same');
  await check('account-same', 'alpha');
  const ws = 'account-fixture-workspace';
  await invoke('create_workspace', { id: ws, name: ws, icon: 'A', accentPrimary: '#4c83d1', accentSecondary: '#8493a8' });
  await invoke('move_tab_to_workspace', { tabId: second.id, workspaceId: ws });
  await check('account-same', 'alpha');
  assert.equal(fixture.loads.get('account-same'), 1, 'moving an account tab must preserve its resident document');
  await invoke('set_workspace_isolation', { enabled: false });
  await invoke('set_workspace_isolation', { enabled: true });
  assert.equal(fixture.loads.get('account-a'), 1, 'isolation preference preserves a hidden account document');
  assert.equal(fixture.loads.get('account-same'), 1, 'isolation preference preserves the active account document');
  const other = await open(b.id, 'account-other');
  await check('account-other', null);
  const normal = (await invoke('create_tab', { url: fixture.origin + '/page/account-default' })).tabs.find(tab => tab.is_active);
  await check('account-default', null);
  fixture.commands.set('account-default', 'store-account-alpha');
  await until(() => fixture.reports.get('account-default')?.account?.indexed === 'alpha', 'seed workspace profile');
  await invoke('move_tab_to_workspace', { tabId: normal.id, workspaceId: 'ws-personal' });
  await check('account-default', null, 2);
  const ephemeral = (await invoke('create_ephemeral_tab', { url: fixture.origin + '/page/account-private' })).tabs.find(tab => tab.is_active);
  await check('account-private', null);
  await assert.rejects(invoke('set_tab_account_container', { tabId: ephemeral.id, containerId: a.id }));
  await invoke('close_tab', { tabId: ephemeral.id });
  await invoke('switch_tab', { tabId: other.id });
  const assignedLoad = fixture.loads.get('account-other') + 1;
  await invoke('set_tab_account_container', { tabId: other.id, containerId: a.id });
  await check('account-other', 'alpha', assignedLoad);
  const restoredLoad = fixture.loads.get('account-other') + 1;
  await restart();
  assert.equal((await invoke('get_tabs')).tabs.find(tab => tab.id === other.id)?.container_id, a.id);
  await check('account-other', 'alpha', restoredLoad);
  const detachedLoad = fixture.loads.get('account-other') + 1;
  await invoke('delete_account_container', { id: a.id });
  const remaining = (await invoke('get_tabs')).tabs;
  assert.equal(remaining.find(tab => tab.id === first.id)?.container_id, null);
  assert.equal(remaining.find(tab => tab.id === other.id)?.container_id, null);
  assert.equal(remaining.find(tab => tab.id === normal.id)?.container_id, null);
  assert.equal(existsSync(join(directory, 'profile', 'webview-accounts', a.id)), false);
  // The workspace still has the artificial data seeded before the normal tab
  // moved out. Removing it must retire even hidden documents using that profile.
  await check('account-other', 'alpha', detachedLoad);
  await invoke('switch_tab', { tabId: first.id });
  const movedLoad = fixture.loads.get('account-other') + 1;
  await invoke('delete_workspace', { workspaceId: ws, action: 'move', targetWorkspaceId: 'ws-personal' });
  await invoke('switch_tab', { tabId: other.id });
  await check('account-other', null, movedLoad);
  assert.equal((await invoke('get_tabs')).tabs.find(tab => tab.id === other.id).workspace_id, 'ws-personal');
}
