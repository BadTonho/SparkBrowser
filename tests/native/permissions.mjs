import assert from 'node:assert/strict';
export async function checkPermissions(invoke, until, fixture, a, b, script, withWebview, click) {
  if (process.platform !== 'win32') return;
  assert.equal((await invoke('get_site_permissions')).supported, true);
  const policy = state => invoke('set_site_permission', { origin: fixture.origin, kind: 'microphone', state });
  const pending = async () => (await invoke('get_site_permissions')).requests.find(row => row.origin === fixture.origin && row.kind === 'microphone');
  let p;
  async function open() {
    fixture.reports.delete('p');
    p = (await invoke('create_tab', { url: fixture.origin + '/page/p', searchEngine: 'google' })).tabs.find(tab => tab.is_active);
    await until(() => fixture.reports.has('p'), 'permission fixture loaded');
  }
  async function request() { fixture.commands.set('p', 'microphone'); return until(pending, 'native microphone permission'); }
  await policy('ask'); await open();
  await click('.omnibox-icon');
  await withWebview('/permission-popover.html', () => until(() => script(`return document.querySelector('#permission-popover')?.dataset.mode === 'settings' && !document.querySelector('#permission-popover').hidden`), 'native current-site permissions panel'));
  await withWebview('/permission-popover.html', async () => {
    await script(`document.querySelector('#close').click(); return true`);
    await until(() => script(`return document.querySelector('#permission-popover').hidden`), 'permissions panel closes');
  });
  let current = await request();
  await withWebview('/permission-popover.html', () => until(() => script(`return document.querySelector('#permission-popover')?.dataset.mode === 'request' && !document.querySelector('#permission-popover').hidden`), 'native permission request popover'));
  await invoke('respond_site_permission', { id: current.id, decision: 'allow_once' });
  await until(() => fixture.reports.get('p')?.permission === 'granted', 'fake microphone granted once');
  assert.equal((await invoke('get_site_permissions')).settings.some(row => row.origin === fixture.origin && row.kind === 'microphone'), false);
  await invoke('close_tab', { tabId: p.id, searchEngine: 'google' }); await open();
  current = await request();
  await invoke('respond_site_permission', { id: current.id, decision: 'block_always' });
  await until(() => fixture.reports.get('p')?.permission === 'denied', 'microphone blocked');
  const blockedSettings = (await invoke('get_site_permissions')).settings;
  assert.ok(blockedSettings.some(row => row.origin === fixture.origin && row.kind === 'microphone' && row.state === 'block'), JSON.stringify(blockedSettings.map(row => ({ kind: row.kind, state: row.state, sameOrigin: new URL(row.origin).origin === fixture.origin }))));
  await policy('ask');
  await invoke('close_tab', { tabId: p.id, searchEngine: 'google' }); await open();
  current = await request();
  fixture.reports.delete('p');
  await invoke('navigate_to', { input: fixture.origin + '/page/p?new=document', searchEngine: 'google' });
  await until(async () => !(await pending()), 'navigation cancels pending request');
  await assert.rejects(invoke('respond_site_permission', { id: current.id, decision: 'allow_once' }));
  await until(() => fixture.reports.get('p')?.url === '/page/p?new=document', 'new permission document');

  current = await request();
  console.log('Checking native permission expiry (60 seconds).');
  await until(async () => !(await pending()), 'permission expires without a response', 70_000);
  await assert.rejects(invoke('respond_site_permission', { id: current.id, decision: 'allow_once' }));
  current = await request();
  await invoke('switch_tab', { tabId: a.id });
  await invoke('hibernate_tab', { tabId: p.id });
  await until(async () => !(await pending()), 'hibernation cancels the pending permission');
  assert.equal((await invoke('get_tabs')).tabs.find(tab => tab.id === p.id).is_hibernated, true);
  await assert.rejects(invoke('respond_site_permission', { id: current.id, decision: 'allow_once' }));
  await invoke('close_tab', { tabId: p.id, searchEngine: 'google' }); await open();

  const childOrigin = fixture.origin.replace('127.0.0.1', 'localhost');
  await invoke('set_site_permission', { origin: childOrigin, kind: 'microphone', state: 'ask' });
  fixture.commands.set('p', 'iframe');
  const child = await until(async () => (await invoke('get_site_permissions')).requests.find(row => row.frame_origin === childOrigin), 'iframe permission origin');
  assert.equal(child.top_origin, fixture.origin);
  await invoke('respond_site_permission', { id: child.id, decision: 'block' });
  await invoke('close_tab', { tabId: p.id, searchEngine: 'google' }); await open();
  current = await request();
  await invoke('respond_site_permission', { id: current.id, decision: 'allow_always' });
  await until(() => fixture.reports.get('p')?.permission === 'granted', 'remembered microphone grant completed');
  assert.ok((await invoke('get_site_permissions')).settings.some(row => row.origin === fixture.origin && row.kind === 'microphone' && row.state === 'allow'));
  await invoke('close_tab', { tabId: p.id, searchEngine: 'google' });
  await invoke('switch_tab', { tabId: a.id });
  await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  await invoke('commit_split_ratio', { key: `${a.workspace_id}:${a.id}:${b.id}`, ratio: 0.65 });
}
