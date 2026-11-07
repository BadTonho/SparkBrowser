import assert from 'node:assert/strict';
import { createNetworkPrivacyFixture } from './network-privacy-fixture.mjs';

// Establishes the unprotected baseline only. This is not a leak-prevention test.
export async function checkNetworkPrivacyBaseline({ invoke, until, withWebview, script }) {
  const fixture = await createNetworkPrivacyFixture();
  try {
    const runtime = await invoke('get_webview_runtime_status');
    console.log(`Native WebView runtime: ${runtime.currentVersion ?? 'unavailable'}`);
    await invoke('create_tab', { url: `${fixture.origin}/page` });
    await withWebview('/page', async () => assert.equal(await script('return document.body.textContent'), 'fixture'));
    await until(() => fixture.httpRequests.includes('/page'), 'loopback fixture observed native WebView request');
    await invoke('navigate_to', { input: `${fixture.origin}/redirect-http` });
    await until(() => fixture.httpRequests.includes('/redirect-http') && fixture.httpRequests.filter(path => path === '/page').length >= 2, 'HTTP redirect observed by loopback fixture');
    assert.deepEqual(fixture.socksRequests, [], 'normal navigation does not claim to use the fixture SOCKS endpoint');
  } finally {
    await fixture.close();
  }
}
