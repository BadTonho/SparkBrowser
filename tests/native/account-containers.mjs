import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

export async function checkAccountContainers({ invoke, fixture, withWebview, script, restart, directory }) {
  const accounts = await invoke('create_account_container', { name: 'Fixture A' });
  const a = accounts.find(item => item.name === 'Fixture A');
  const b = (await invoke('create_account_container', { name: 'Fixture B' })).find(item => item.name === 'Fixture B');
  assert.ok(a && b && a.id !== b.id);
  await invoke('set_workspace_isolation', { enabled: true });
  const open = async (id, path) => {
    const snapshot = await invoke('create_container_tab', { id });
    const tab = snapshot.tabs.find(item => item.is_active);
    await invoke('navigate_to', { input: fixture.origin + path });
    return tab;
  };
  const first = await open(a.id, '/page/account-a');
  await withWebview('/page/account-a', async () => {
    await script(`localStorage.setItem('fixture-account', 'alpha'); document.cookie='fixture_account=alpha; Max-Age=3600; Path=/'; return null;`);
    await script(`return new Promise((resolve,reject)=>{ const q=indexedDB.open('fixture-account',1); q.onupgradeneeded=()=>q.result.createObjectStore('records'); q.onerror=()=>reject(q.error); q.onsuccess=()=>{const tx=q.result.transaction('records','readwrite'); tx.objectStore('records').put('alpha','value'); tx.oncomplete=()=>{q.result.close(); resolve(true)}; tx.onerror=()=>reject(tx.error)}; });`);
  });
  const second = await open(a.id, '/page/account-same');
  const ws = 'account-fixture-workspace';
  await invoke('create_workspace', { id: ws, name: ws, icon: 'A', accentPrimary: '#4c83d1', accentSecondary: '#8493a8' });
  await invoke('move_tab_to_workspace', { tabId: second.id, workspaceId: ws });
  await withWebview('/page/account-same', async () => {
    assert.equal(await script(`return localStorage.getItem('fixture-account')`), 'alpha');
    assert.match(await script(`return document.cookie`), /fixture_account=alpha/);
    assert.equal(await script(`return new Promise((resolve,reject)=>{ const q=indexedDB.open('fixture-account',1); q.onerror=()=>reject(q.error); q.onsuccess=()=>{const tx=q.result.transaction('records'); const read=tx.objectStore('records').get('value'); read.onsuccess=()=>{resolve(read.result);q.result.close()}; read.onerror=()=>reject(read.error)}; });`), 'alpha');
  });
  const other = await open(b.id, '/page/account-other');
  await withWebview('/page/account-other', async () => {
    assert.equal(await script(`return localStorage.getItem('fixture-account')`), null);
    assert.doesNotMatch(await script(`return document.cookie`), /fixture_account=alpha/);
  });
  const normal = (await invoke('create_tab', { url: fixture.origin + '/page/account-default' })).tabs.find(tab => tab.is_active);
  await withWebview('/page/account-default', async () => {
    assert.equal(await script(`return localStorage.getItem('fixture-account')`), null);
    assert.doesNotMatch(await script(`return document.cookie`), /fixture_account=alpha/);
  });
  const ephemeral = (await invoke('create_ephemeral_tab', { url: fixture.origin + '/page/account-private' })).tabs.find(tab => tab.is_active);
  await withWebview('/page/account-private', async () => assert.equal(await script(`return localStorage.getItem('fixture-account')`), null));
  await assert.rejects(invoke('set_tab_account_container', { tabId: ephemeral.id, containerId: a.id }));
  await invoke('close_tab', { tabId: ephemeral.id });
  await invoke('switch_tab', { tabId: other.id });
  await invoke('set_tab_account_container', { tabId: other.id, containerId: a.id });
  await withWebview('/page/account-other', async () => assert.equal(await script(`return localStorage.getItem('fixture-account')`), 'alpha'));
  await restart();
  assert.equal((await invoke('get_tabs')).tabs.find(tab => tab.id === other.id)?.container_id, a.id);
  await withWebview('/page/account-other', async () => assert.equal(await script(`return localStorage.getItem('fixture-account')`), 'alpha'));
  await invoke('delete_account_container', { id: a.id });
  const remaining = (await invoke('get_tabs')).tabs;
  assert.equal(remaining.find(tab => tab.id === first.id)?.container_id, null);
  assert.equal(remaining.find(tab => tab.id === other.id)?.container_id, null);
  assert.equal(remaining.find(tab => tab.id === normal.id)?.container_id, null);
  assert.equal(existsSync(join(directory, 'profile', 'webview-accounts', a.id)), false);
}
