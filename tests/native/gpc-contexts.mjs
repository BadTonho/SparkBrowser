import assert from 'node:assert/strict';
export async function checkGpcContexts({ invoke, fixture, until }) {
  await invoke('set_gpc', { settings: { enabled: true, exceptions: [] } });
  await invoke('navigate_to', { input: `${fixture.origin}/page/gpc-contexts` });
  const paths = ['/gpc-resource/frame', '/gpc-resource/main-fetch', '/gpc-resource/frame-fetch', '/gpc-resource/worker.js', '/gpc-resource/worker-fetch'];
  await until(() => paths.every(path => fixture.gpcRequests.some(item => item.url === path)), 'GPC frame and worker requests');
  if (process.platform === 'win32') for (const path of paths) assert.equal(fixture.gpcRequests.find(item => item.url === path).header, '1', `GPC header on ${path}`);
  fixture.gpcRequests.length = 0;
  await invoke('set_gpc', { settings: { enabled: true, exceptions: ['127.0.0.1'] } });
  await invoke('navigate_to', { input: `${fixture.origin}/page/gpc-contexts` });
  await until(() => paths.every(path => fixture.gpcRequests.some(item => item.url === path)), 'GPC exception frame and worker requests');
  for (const path of paths) assert.equal(fixture.gpcRequests.find(item => item.url === path).header, undefined, `excluded GPC header on ${path}`);
}
