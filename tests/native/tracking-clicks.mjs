import assert from 'node:assert/strict';

export async function checkTrackingClicks({ invoke, fixture, until, withWebview, click }) {
  await invoke('set_tracking_cleaning', { settings: { enabled: true, exceptions: [] } });
  await invoke('navigate_to', { input: `${fixture.origin}/page/link-source` });
  await withWebview('/page/link-source', () => click('#tracking-link'));
  await until(() => fixture.trackingRequests.some(request => request.url.startsWith('/page/link-target')), 'clicked link reached destination');
  assert.deepEqual(fixture.trackingRequests.find(request => request.url.startsWith('/page/link-target')), { url: '/page/link-target?keep=1', method: 'GET' });
  await invoke('navigate_to', { input: `${fixture.origin}/page/link-source` });
  await withWebview('/page/link-source', () => click('#post-button'));
  await until(() => fixture.trackingRequests.some(request => request.url.startsWith('/page/post-target')), 'POST reached destination');
  assert.deepEqual(fixture.trackingRequests.find(request => request.url.startsWith('/page/post-target')), { url: '/page/post-target?utm_source=fixture', method: 'POST' });
  if (process.platform === 'win32') {
    await invoke('navigate_to', { input: `${fixture.origin}/page/tracking-redirect` });
    await until(() => fixture.trackingRequests.some(request => request.url.includes('keep=redirect')), 'redirect destination reached');
    assert.deepEqual(fixture.trackingRequests.find(request => request.url.includes('keep=redirect')), { url: '/page/link-target?keep=redirect', method: 'GET' });
    await invoke('navigate_to', { input: `${fixture.origin}/page/signed-redirect` });
    await until(() => fixture.trackingRequests.some(request => request.url.includes('state=opaque')), 'signed redirect destination reached');
    assert.equal(fixture.trackingRequests.find(request => request.url.includes('state=opaque')).url, '/page/link-target?utm_source=signed&state=opaque');
    await invoke('navigate_to', { input: `${fixture.origin}/page/link-source` });
    await withWebview('/page/link-source', () => click('#post-redirect-button'));
    await until(() => fixture.trackingRequests.some(request => request.url.includes('utm_source=post-redirect')), '307 POST destination reached');
    assert.equal(fixture.trackingRequests.find(request => request.url.includes('utm_source=post-redirect')).method, 'POST');
  }
}
