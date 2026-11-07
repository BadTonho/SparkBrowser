import assert from 'node:assert/strict';

export async function checkTrackingCleaning({ invoke, fixture, restart }) {
  assert.deepEqual(await invoke('get_tracking_cleaning'), { enabled: false, exceptions: [] });
  const input = `${fixture.origin}/page/tracking?utm_source=fixture&keep=%2F`;
  assert.equal(await invoke('navigate_to', { input }), input);
  await invoke('set_tracking_cleaning', { settings: { enabled: true, exceptions: [] } });
  assert.equal(await invoke('navigate_to', { input }), `${fixture.origin}/page/tracking?keep=%2F`);
  const sensitive = `${fixture.origin}/page/tracking?utm_source=fixture&state=opaque`;
  assert.equal(await invoke('navigate_to', { input: sensitive }), sensitive);
  await invoke('set_tracking_cleaning', { settings: { enabled: true, exceptions: ['127.0.0.1'] } });
  assert.equal(await invoke('navigate_to', { input }), input);
  await restart();
  assert.deepEqual(await invoke('get_tracking_cleaning'), { enabled: true, exceptions: ['127.0.0.1'] });
  assert.equal(await invoke('navigate_to', { input }), input);
}
