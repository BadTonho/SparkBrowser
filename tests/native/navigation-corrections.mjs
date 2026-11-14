import assert from 'node:assert/strict';

export async function checkFullscreenCorrection(context) {
  const { invoke, fixture, until, withWebview, script, click, request, sessionId, focusWindow } = context;
  await invoke('navigate_to', { input: fixture.origin + '/page/fullscreen-corrections' });
  await until(() => fixture.reports.has('fullscreen-corrections'), 'fullscreen fixture');
  const source = (await invoke('get_tabs')).tabs.find(t => t.is_active);
  await focusWindow();
  await withWebview('/page/fullscreen-corrections', async () => {
    await script(`document.body.insertAdjacentHTML('afterbegin','<button id="fullscreen-test">Fullscreen</button><video id="fullscreen-video"></video>');document.querySelector('#fullscreen-test').onclick=()=>document.querySelector('#fullscreen-video').requestFullscreen();`);
    await click('#fullscreen-test');
    await until(() => script('return Boolean(document.fullscreenElement)'), 'page enters fullscreen');
    await request('POST', `/session/${sessionId()}/actions`, { actions: [{ type: 'key', id: 'esc', actions: [{ type: 'keyDown', value: '\uE00C' }, { type: 'keyUp', value: '\uE00C' }] }] });
    await until(() => script('return !document.fullscreenElement'), 'Escape exits fullscreen');
    await click('#fullscreen-test');
    await until(() => script('return Boolean(document.fullscreenElement)'), 'page re-enters fullscreen');
  });
  const next = (await invoke('create_tab', { url: fixture.origin + '/page/fullscreen-next' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('fullscreen-next'), 'new tab after fullscreen');
  await invoke('switch_tab', { tabId: source.id });
  await withWebview('/page/fullscreen-corrections', async () => {
    assert.equal(await script('return Boolean(document.fullscreenElement)'), false, 'changing tabs exits fullscreen');
  });
  assert.notEqual(source.id, next.id);
}

// The fixture serves only loopback content and synthetic paths.
export async function checkNavigationCorrections(context) {
  const { invoke, fixture, until, withWebview, script, click, request, sessionId, focusWindow, wait, restart } = context;
  await invoke('set_popup_setting', { enabled: false });
  await invoke('navigate_to', { input: fixture.origin + '/page/corrections' });
  await until(() => fixture.reports.has('corrections'), 'navigation fixture');
  const source = (await invoke('get_tabs')).tabs.find(t => t.is_active);
  await focusWindow();
  await withWebview('/page/corrections', async () => {
    await script(`document.body.insertAdjacentHTML('afterbegin', '<a id="test-link" href="/page/background">Background</a><button id="test-fullscreen">Fullscreen</button><video id="test-video"></video>'); document.querySelector('#test-fullscreen').onclick=()=>document.querySelector('#test-video').requestFullscreen();`);
    const node = await request('POST', `/session/${sessionId()}/element`, { using: 'css selector', value: '#test-link' });
    const element = node['element-6066-11e4-a52e-4f735466cecf'];
    await request('POST', `/session/${sessionId()}/actions`, { actions: [
      { type: 'key', id: 'modifier', actions: [{ type: 'keyDown', value: '\uE009' }, { type: 'pause', duration: 200 }, { type: 'pause', duration: 100 }, { type: 'keyUp', value: '\uE009' }] },
      { type: 'pointer', id: 'mouse', parameters: { pointerType: 'mouse' }, actions: [{ type: 'pointerMove', origin: { 'element-6066-11e4-a52e-4f735466cecf': element }, x: 0, y: 0 }, { type: 'pointerDown', button: 0 }, { type: 'pointerUp', button: 0 }] }
    ] });
  });
  await until(async () => (await invoke('get_tabs')).tabs.some(t => t.url.endsWith('/page/background')), 'background link tab');
  const tabs = (await invoke('get_tabs')).tabs;
  const opened = tabs.find(t => t.url.endsWith('/page/background'));
  assert.notEqual(opened.id, source.id);
  assert.equal(tabs.find(t => t.is_active).id, source.id);
  assert.equal(opened.workspace_id, source.workspace_id);
  assert.equal(opened.container_id, source.container_id);
  assert.equal(opened.is_ephemeral, source.is_ephemeral);
  await withWebview('/page/corrections', async () => {
    await script('document.querySelector("#test-link").href="/page/middle-background"');
    const node = await request('POST', `/session/${sessionId()}/element`, { using: 'css selector', value: '#test-link' });
    await request('POST', `/session/${sessionId()}/actions`, { actions: [{ type: 'pointer', id: 'middle', parameters: { pointerType: 'mouse' }, actions: [
      { type: 'pointerMove', origin: { 'element-6066-11e4-a52e-4f735466cecf': node['element-6066-11e4-a52e-4f735466cecf'] }, x: 0, y: 0 },
      { type: 'pointerDown', button: 1 }, { type: 'pointerUp', button: 1 }
    ] }] });
  });
  await until(async () => (await invoke('get_tabs')).tabs.some(t => t.url.endsWith('/page/middle-background') && t.id !== source.id), 'middle-click background tab');
  await withWebview('/page/corrections', () => script('window.open("/page/automatic-popup")'));
  await wait(250);
  assert.ok(!(await invoke('get_tabs')).tabs.some(t => t.url.endsWith('/page/automatic-popup')), 'automatic popup remains blocked');
  await withWebview('/page/corrections', async () => {
    await click('#test-fullscreen');
    await until(() => script('return Boolean(document.fullscreenElement)'), 'page enters fullscreen');
    await request('POST', `/session/${sessionId()}/actions`, { actions: [{ type: 'key', id: 'esc', actions: [{ type: 'keyDown', value: '\uE00C' }, { type: 'keyUp', value: '\uE00C' }] }] });
    await until(() => script('return !document.fullscreenElement'), 'Escape exits fullscreen');
  });
  await invoke('set_history_retention', { retention: 'never' });
  await invoke('navigate_to', { input: fixture.origin + '/page/never-record' });
  await until(() => fixture.reports.has('never-record'), 'unrecorded visit');
  await wait(250);
  assert.ok(!(await invoke('get_history', { limit: 100 })).some(entry => entry.url.endsWith('/page/never-record')));
  await restart();
  assert.equal((await invoke('get_privacy_settings')).history_retention, 'never');
  await script(`localStorage.setItem('spark_keyboard_shortcuts',JSON.stringify({zen:[{key:'Z',ctrl:false,alt:true,shift:false,meta:false}],history:[{key:'Y',ctrl:true,alt:false,shift:true,meta:false}]}));`);
  await restart();
  const migrated = await script('return JSON.parse(localStorage.getItem("spark_keyboard_shortcuts"))');
  assert.deepEqual(migrated.zen[0], { key:'Z', ctrl:true, alt:false, shift:true, meta:false });
  assert.equal(migrated.history[0].key, 'Y', 'custom shortcut survives migration');
}
