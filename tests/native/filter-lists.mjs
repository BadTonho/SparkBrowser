import assert from 'node:assert/strict';

export async function checkFilterLists({ invoke, fixture, until, restart }) {
  assert.deepEqual(await invoke('get_shield_exceptions'), []);
  await invoke('navigate_to', { input: `${fixture.origin}/page/filter-test` });
  await until(() => fixture.shieldRequests.includes('/ordinary-script.js'), 'ordinary script reaches loopback server');
  if (process.platform === 'win32') assert.ok(!fixture.shieldRequests.includes('/ads-native.js'), 'EasyList script should be intercepted before reaching the server');
  await invoke('set_shield_exceptions', { exceptions: ['127.0.0.1'] });
  await invoke('navigate_to', { input: `${fixture.origin}/page/filter-test` });
  await until(() => fixture.shieldRequests.includes('/ads-native.js'), 'excepted site loads its script');
  await restart();
  assert.deepEqual(await invoke('get_shield_exceptions'), ['127.0.0.1']);
}
