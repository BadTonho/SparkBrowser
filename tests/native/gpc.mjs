import assert from 'node:assert/strict';

export async function checkGpc({ invoke, fixture, until, restart, withWebview, script }) {
  assert.deepEqual(await invoke('get_gpc'), { enabled: false, exceptions: [] });
  await invoke('navigate_to', { input: `${fixture.origin}/page/gpc-off` });
  await until(() => fixture.gpcRequests.some(item => item.url.startsWith('/page/gpc-off')), 'GPC disabled request');
  assert.equal(fixture.gpcRequests.find(item => item.url.startsWith('/page/gpc-off')).header, undefined);
  assert.notEqual(await withWebview('/page/gpc-off', () => script('return navigator.globalPrivacyControl')), true);
  await invoke('set_gpc', { settings: { enabled: true, exceptions: [] } });
  await invoke('navigate_to', { input: `${fixture.origin}/page/gpc-on` });
  await until(() => fixture.gpcRequests.some(item => item.url.startsWith('/page/gpc-on')), 'GPC enabled request');
  if (process.platform === 'win32') assert.equal(fixture.gpcRequests.find(item => item.url.startsWith('/page/gpc-on')).header, '1');
  assert.equal(await withWebview('/page/gpc-on', () => script('return navigator.globalPrivacyControl')), true);
  await invoke('set_gpc', { settings: { enabled: true, exceptions: ['127.0.0.1'] } });
  await invoke('navigate_to', { input: `${fixture.origin}/page/gpc-excluded` });
  await until(() => fixture.gpcRequests.some(item => item.url.startsWith('/page/gpc-excluded')), 'GPC exception request');
  assert.equal(fixture.gpcRequests.find(item => item.url.startsWith('/page/gpc-excluded')).header, undefined);
  assert.notEqual(await withWebview('/page/gpc-excluded', () => script('return navigator.globalPrivacyControl')), true);
  await restart();
  assert.deepEqual(await invoke('get_gpc'), { enabled: true, exceptions: ['127.0.0.1'] });
}
