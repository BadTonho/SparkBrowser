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
}
