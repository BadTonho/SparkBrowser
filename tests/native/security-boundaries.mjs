import assert from 'node:assert/strict';

export async function checkSecurityBoundaries(context) {
  const { script, invoke, fixture, until, withWebview, click, wait, request, sessionId } = context;
  await context.focusWindow();
  const shellUrl = await request('GET', `/session/${sessionId()}/url`);
  await script('history.pushState({},"","/index.html?unexpected=document");');
  await assert.rejects(() => invoke('get_tabs'), /restricted|not allowed|denied/i);
  await script('history.replaceState({},"",arguments[0])', shellUrl);
  assert.ok((await invoke('get_tabs')).tabs.length > 0);
  await script('location.href="https://external-navigation.invalid/"'); await wait(100);
  assert.equal(await request('GET', `/session/${sessionId()}/url`), shellUrl, 'main cannot navigate externally');
  const windows = (await request('GET', `/session/${sessionId()}/window/handles`)).length;
  await script('window.open("https://external-navigation.invalid/")'); await wait(100);
  assert.equal((await request('GET', `/session/${sessionId()}/window/handles`)).length, windows);
  await invoke('navigate_to', { input: fixture.origin + '/page/security' });
  await until(() => fixture.reports.has('security'), 'untrusted content loaded');
  await withWebview('/page/security', async () => {
    await assert.rejects(() => invoke('get_tabs'), /restricted|not allowed|denied/i);
    if (process.platform === 'win32') await script(`window.__sparkPostContent('spark-internal://page-activity/state?token=invalid&document=fixture&sequence=999&dirty=true&media=true');window.__sparkPostContent('spark-internal://new-tab/navigate?token=invalid&document=fixture&sequence=999&input=https%3A%2F%2Fexternal-navigation.invalid');`);
    await wait(100);
    await script('location.href="http://tauri.localhost/index.html"'); await wait(100);
    assert.ok((await request('GET', `/session/${sessionId()}/url`)).startsWith(fixture.origin));
    await script('location.href="file:///C:/nonexistent-security-fixture.html"'); await wait(100);
    assert.ok((await request('GET', `/session/${sessionId()}/url`)).startsWith(fixture.origin));
  });
  const untrusted = (await invoke('get_tabs')).tabs.find(tab => tab.url.startsWith(fixture.origin + '/page/security'));
  assert.ok(untrusted && !untrusted.activity.dirty_form && !untrusted.activity.playing_media, 'unauthenticated native messages cannot change session observations');

  await click('[data-testid=open-privacy]');
  const before = (await invoke('get_privacy_stats')).shield_enabled;
  await withWebview('/privacy-popover.html', async () => {
    await until(() => script('return !document.querySelector("#privacy-popover").hidden'), 'privacy panel');
    await script(`window.originalUrlString=URL.prototype.toString;URL.prototype.toString=function(){window.capturedBridge=window.originalUrlString.call(this);return window.capturedBridge+'&token=duplicate'};document.querySelector('#adblock').click();`);
    await wait(100);
    await script('URL.prototype.toString=window.originalUrlString');
  });
  assert.equal((await invoke('get_privacy_stats')).shield_enabled, before, 'a duplicated decoded parameter cannot mutate protection');
  for (const mutate of [
    'u.searchParams.set("value","invalid")',
    'u.searchParams.delete("value")',
    'u.searchParams.set("command","close_tab")',
    'u.pathname="/unknown"',
    'u.searchParams.set("sequence","0")',
    'u.searchParams.set("token","invalid")',
  ]) {
    await withWebview('/privacy-popover.html', () => script(`const u=new URL(window.capturedBridge);${mutate};location.href=u.toString();`));
    await wait(100);
    assert.equal((await invoke('get_privacy_stats')).shield_enabled, before, 'invalid action is inert');
  }
  await withWebview('/privacy-popover.html', () => script('location.href=window.capturedBridge'));
  await until(async () => (await invoke('get_privacy_stats')).shield_enabled !== before, 'valid authenticated action');
  await withWebview('/privacy-popover.html', () => script('const u=new URL(window.capturedBridge);u.searchParams.set("value",arguments[0]);location.href=u.toString();', String(before)));
  await wait(100);
  assert.equal((await invoke('get_privacy_stats')).shield_enabled, !before, 'replayed sequence cannot change protection');
  await withWebview('/privacy-popover.html', () => script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))'));

  await until(() => script('return document.querySelector("[data-testid=open-privacy]").getAttribute("aria-expanded") !== "true"'), 'privacy dismissed');
  await click('[data-testid=open-privacy]');
  await withWebview('/privacy-popover.html', async () => {
    await until(() => script('return !document.querySelector("#privacy-popover").hidden'), 'new privacy session');
    await script('const u=new URL(window.capturedBridge);u.searchParams.set("sequence","999");u.searchParams.set("value",arguments[0]);location.href=u.toString();', String(before));
    await wait(100);
    await script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))');
  });
  assert.equal((await invoke('get_privacy_stats')).shield_enabled, !before, 'closed session token cannot act on a reopened panel');
  await until(() => script('return document.querySelector("[data-testid=open-privacy]").getAttribute("aria-expanded") !== "true"'), 'second privacy dismissed');
  await invoke('toggle_adblock_shield');
  // Create native internal documents through the normal browser path.
  await click('[data-action=tools]');
  await withWebview('/index.html?surface=tools', async () => {
    await until(() => script('return !!document.querySelector("[data-testid=tools-dialog]")'), 'tools panel');
    await assert.rejects(() => invoke('get_tabs'), /restricted|not allowed|denied/i);
    await script('location.href="https://external-navigation.invalid/"'); await wait(100);
    assert.equal(new URL(await request('GET', `/session/${sessionId()}/url`)).search, '?surface=tools');
    await script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))');
  });
}
