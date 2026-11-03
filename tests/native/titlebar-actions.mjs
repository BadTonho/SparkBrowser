import assert from 'node:assert/strict';

export async function checkTitlebarActions(context) {
  const { script, request, sessionId, until, click, restart, screenshot, invoke, fixture, resizeWindow, withWebview } = context;
  const order = () => script('return [...document.querySelectorAll(".titlebar-actions [data-action]")].map(b => b.dataset.action)');
  const saved = () => script('return localStorage.getItem("veloce_titlebar_action_order")');
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
  const defaults = ['tools', 'bookmarks', 'commands', 'history', 'downloads', 'privacy', 'settings', 'zen'];
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
    { type: 'keyDown', value: '\uE00A' }, { type: 'keyDown', value: 'z' },
    { type: 'keyUp', value: 'z' }, { type: 'keyUp', value: '\uE00A' },
  ] }] });
  await until(() => script('return !!document.querySelector(".titlebar-actions")'), 'Alt+Z restores toolbar');
  for (const [id, selector] of [['tools', '[data-testid=tools-dialog]'], ['commands', '.command-modal'], ['history', '.history-panel'], ['downloads', '.downloads-panel'], ['settings', '.settings-modal']]) {
    await click(`[data-action="${id}"]`);
    await until(() => script('return !!document.querySelector(arguments[0])', selector), id + ' quick click opens');
    await escape();
    await until(() => script('return !document.querySelector(arguments[0])', selector), id + ' closes');
  }
  await click('[data-action="privacy"]');
  await until(() => script('return document.querySelector("[data-action=privacy]").getAttribute("aria-expanded")==="true"'), 'privacy still opens');
  await withWebview('/privacy-popover.html', async () => {
    await script('window.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}))');
  });
  await until(() => script('return document.querySelector("[data-action=privacy]").getAttribute("aria-expanded")==="false"'), 'privacy closes');

  // Switch through the real settings UI, keeping the hidden bookmark slot intact.
  await click('[data-action="settings"]'); await click('[data-testid="settings-appearance_layout"]');
  await click('#setting-browser-layout .mini-pill:nth-child(2)'); await escape();
  await until(async () => (await order()).length === 7, 'vertical layout hides bookmark action');
  const verticalRect = await request('GET', `/session/${sessionId()}/window/rect`);
  await gesture('zen','left');
  const expected = ['zen', 'bookmarks', 'tools', 'commands', 'history', 'downloads', 'privacy', 'settings'];
  assert.deepEqual(JSON.parse(await saved()), expected);
  assert.deepEqual(await request('GET', `/session/${sessionId()}/window/rect`), verticalRect, 'vertical drag region does not move the window');
  await click('[data-action="settings"]');
  await click('#setting-browser-layout .mini-pill:first-child'); await escape();
  await until(async () => (await order()).length === 8, 'horizontal restores bookmark action');
  assert.deepEqual(await order(), expected);
  await restart(); assert.deepEqual(await order(), expected, 'order survives native app restart');

  await invoke('navigate_to', { input: fixture.origin + '/page/toolbar-a' });
  const a = (await invoke('get_tabs')).tabs.find(tab => tab.is_active);
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/toolbar-b' })).tabs.find(tab => tab.is_active);
  await invoke('set_split_view', { leftTabId: a.id, rightTabId: b.id });
  await resizeWindow(900, 700);
  await until(() => script('return !!document.querySelector("[data-testid=open-titlebar-menu]")'), 'compact menu replaces actions');
  assert.equal(await script('return !!document.querySelector(".titlebar-actions")'), false);
  await click('[data-testid=open-titlebar-menu]');
  await until(() => script('return !!document.querySelector("[data-testid=titlebar-menu]")'), 'compact menu opens');
  await escape();
  await resizeWindow(1280, 820);
  await until(async () => (await order()).length === 8, 'expanding restores actions');
  assert.deepEqual(await order(), expected);
  await screenshot('titlebar-reorder-saved');
}
