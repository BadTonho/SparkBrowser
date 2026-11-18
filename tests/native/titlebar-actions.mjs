import assert from 'node:assert/strict';

export async function checkTitlebarActions(context) {
  await context.focusWindow();
  const { script, request, sessionId, until, click, restart, screenshot, invoke, fixture, resizeWindow, withWebview } = context;
  const order = () => script('return [...document.querySelectorAll(".titlebar-actions [data-action]")].map(b => b.dataset.action)');
  const saved = () => script('return localStorage.getItem("spark_titlebar_action_order_v2")');
  const actions = list => request('POST', `/session/${sessionId()}/actions`, { actions: [{ type: 'pointer', id: 'toolbar-mouse', parameters: { pointerType: 'mouse' }, actions: list }] });
  const move = (x, y) => ({ type: 'pointerMove', origin: 'viewport', x: Math.round(x), y: Math.round(y), duration: 100 });
  const key = value => request('POST', `/session/${sessionId()}/actions`, { actions: [{ type: 'key', id: 'toolbar-key', actions: [{ type: 'keyDown', value }, { type: 'keyUp', value }] }] });
  const escape = () => key('\uE00C');
  async function geometry(id) {
    return script(`const b=document.querySelector('[data-action="'+arguments[0]+'"]'), r=b.getBoundingClientRect(), strip=b.parentElement.getBoundingClientRect();
      return {x:r.x+r.width/2,y:r.y+r.height/2,left:strip.left+1,right:strip.right-1,bottom:strip.bottom};`, id);
  }
  async function gesture(id, destination, cancel) {
    // Keep the held gesture in one WebDriver action sequence.
    const g = await geometry(id);
    const x = destination === 'left' ? g.left : g.right;
    const mouse = [move(g.x,g.y), {type:'pointerDown',button:0}, {type:'pause',duration:650}, move(x,cancel === 'outside' ? g.bottom+15 : g.y), {type:'pause',duration:300}, {type:'pointerUp',button:0}];
    const inputs=[{type:'pointer',id:'toolbar-mouse',parameters:{pointerType:'mouse'},actions:mouse}];
    if(cancel === 'escape') inputs.push({type:'key',id:'toolbar-key',actions:[{type:'pause',duration:900},{type:'keyDown',value:'\uE00C'},{type:'keyUp',value:'\uE00C'}]});
    if(cancel === 'blur') await script('setTimeout(()=>window.dispatchEvent(new Event("blur")),900)');
    await request('POST', `/session/${sessionId()}/actions`, {actions:inputs});
  }
  // The buttons the harness turns on, in the browser's default order (the default order puts the less used ones after the first four).
  let defaults = ['tools', 'bookmarks', 'commands', 'history', 'zen', 'qr', 'extensions', 'privacy', 'downloads', 'settings'];
  assert.deepEqual(await order(), defaults);
  await script(`window.toolbarMarkerSeen=false;window.toolbarHoldSeen=false;new MutationObserver(()=>{window.toolbarMarkerSeen ||= !!document.querySelector('.insertion-marker');window.toolbarHoldSeen ||= !!document.querySelector('.dragging')}).observe(document.querySelector('.titlebar-actions'),{subtree:true,attributes:true,childList:true});`);
  const initialRect = await request('GET', `/session/${sessionId()}/window/rect`);
  await gesture('tools','right');
  await until(async () => (await order()).at(-1) === 'tools', 'move first action to end');
  assert.equal(await script('return window.toolbarMarkerSeen && window.toolbarHoldSeen'),true,'hold highlight and insertion marker appear');
  assert.deepEqual(await request('GET', `/session/${sessionId()}/window/rect`), initialRect, 'drag does not move the native window');
  assert.equal(await script('return !!document.querySelector("[data-testid=tools-dialog]")'), false, 'drag must not activate tools');
  await gesture('tools','left');
  assert.deepEqual(await order(), defaults);
  await gesture('qr','left');
  await until(async () => (await order())[0] === 'qr', 'move QR Code action to the start');
  await gesture('qr','right');
  await until(async () => (await order()).at(-1) === 'qr', 'move QR Code action to the end');
  // QR Code sits in the middle by default, so dragging it to the end changes the order; the rest of the scenario starts from it.
  defaults = [...defaults.filter(action => action !== 'qr'), 'qr'];
  assert.deepEqual(await order(), defaults);

  for (const mode of ['escape', 'outside', 'blur']) {
    const before = await saved();
    await gesture('settings','left',mode);
    assert.deepEqual(await order(), defaults, mode + ' cancels without moving');
    assert.equal(await saved(), before);
    assert.equal(await script('return !!document.querySelector(".settings-modal")'), false, 'cancel does not click settings');
  }
  const g = await geometry('settings');
  await actions([move(g.x, g.y), { type: 'pointerDown', button: 0 }, move(g.x - 10, g.y), { type: 'pause', duration: 600 }, { type: 'pointerUp', button: 0 }]);
  assert.equal(await script('return !!document.querySelector(".settings-modal")'), false, 'early movement suppresses click');

  // Quick clicks still activate the original handlers after drag/cancellation.
  const bookmarksPressed = () => script('return document.querySelector("[data-action=bookmarks]").getAttribute("aria-pressed")');
  const originalPressed = await bookmarksPressed();
  await click('[data-action=bookmarks]');
  assert.notEqual(await bookmarksPressed(), originalPressed);
  await click('[data-action=bookmarks]');
  assert.equal(await bookmarksPressed(), originalPressed);
  await click('[data-action=zen]');
  await until(() => script('return !document.querySelector(".titlebar")'), 'zen quick click hides toolbar');
  await request('POST', `/session/${sessionId()}/actions`, { actions: [{ type: 'key', id: 'toolbar-key', actions: [
    { type: 'keyDown', value: '\uE009' }, { type: 'keyDown', value: '\uE008' }, { type: 'keyDown', value: 'z' },
    { type: 'keyUp', value: 'z' }, { type: 'keyUp', value: '\uE008' }, { type: 'keyUp', value: '\uE009' },
  ] }] });
  await until(() => script('return !!document.querySelector(".titlebar-actions")'), 'Ctrl+Shift+Z restores toolbar');
  for (const [id, path] of [['tools', '/index.html?surface=tools'], ['commands', null], ['history', '/history-overlay.html'], ['downloads', '/index.html?surface=downloads'], ['settings', '/index.html?surface=settings-quick']]) {
    await click(`[data-action="${id}"]`);
    const inside = action => path ? withWebview(path, action) : action();
    const visible = () => script('return [...document.querySelectorAll("[role=dialog]")].some(node=>node.getClientRects().length&&!node.hidden)');
    await inside(() => until(visible, id + ' quick click opens'));
    await inside(() => script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true,cancelable:true}))'));
    await inside(() => until(async () => !await visible(), id + ' closes'));
  }
  await click('[data-action="privacy"]');
  await until(() => script('return document.querySelector("[data-action=privacy]").getAttribute("aria-expanded")==="true"'), 'privacy still opens');
  await withWebview('/privacy-popover.html', async () => {
    await script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))');
  });
  await until(() => script('return document.querySelector("[data-action=privacy]").getAttribute("aria-expanded")==="false"'), 'privacy closes');

  // Switch through the real settings UI, keeping the hidden bookmark slot intact.
  await context.openSettings(); await click('[data-testid="settings-appearance_layout"]');
  await click('#setting-browser-layout .mini-pill:nth-child(2)'); await context.closeSettings();
  await until(async () => (await order()).length === 9, 'vertical layout hides bookmark action');
  const verticalRect = await request('GET', `/session/${sessionId()}/window/rect`);
  await gesture('zen','left');
  // The saved order lists every button; the ones that are off (WhatsApp, Resources) keep their slots and are not on the toolbar.
  const savedOrder = ['zen', 'bookmarks', 'tools', 'commands', 'history', 'extensions', 'whatsapp', 'resources', 'privacy', 'downloads', 'settings', 'qr'];
  const expected = savedOrder.filter(action => !['whatsapp', 'resources'].includes(action));
  assert.deepEqual(JSON.parse(await saved()), savedOrder);
  assert.deepEqual(await request('GET', `/session/${sessionId()}/window/rect`), verticalRect, 'vertical drag region does not move the window');
  await context.openSettings();
  await click('#setting-browser-layout .mini-pill:first-child'); await context.closeSettings();
  await until(async () => (await order()).length === 10, 'horizontal restores bookmark action');
  assert.deepEqual(await order(), expected);
  await restart(); assert.deepEqual(await order(), expected, 'order survives native app restart');

  await invoke('navigate_to', { input: fixture.origin + '/page/toolbar-a' });
  await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.is_active)?.url === fixture.origin + '/page/toolbar-a', 'toolbar destination confirmed');
  const a = (await invoke('get_tabs')).tabs.find(tab => tab.is_active);
  await click('[data-action="qr"]');
  await until(() => script('return !!document.querySelector(".panel img[alt]")'), 'QR image generates for active HTTP page');
  assert.equal(await script('return document.querySelector(".panel .url").textContent'), a.url, 'QR panel displays the active tab URL');
  assert.equal(await script('return document.querySelector(".panel img").src.startsWith("data:image/png;base64,")'), true, 'QR output is a PNG data URL');
  assert.deepEqual(await script('return [...document.querySelectorAll(".panel footer button")].map(button => button.disabled)'), [false, false], 'copy and save are available after generation');
  await escape();
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/toolbar-b' })).tabs.find(tab => tab.is_active);
  await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  await resizeWindow(900, 700);
  await until(() => script('return !!document.querySelector("[data-testid=open-titlebar-menu]")'), 'compact menu replaces actions');
  assert.equal(await script('return !!document.querySelector(".titlebar-actions")'), false);
  await click('[data-testid=open-titlebar-menu]');
  await until(() => script('return !!document.querySelector("[data-testid=titlebar-menu]")'), 'compact menu opens');
  await escape();
  await resizeWindow(1280, 820);
  await until(async () => (await order()).length === 10, 'expanding restores actions');
  assert.deepEqual(await order(), expected);
  await screenshot('titlebar-reorder-saved');
}
