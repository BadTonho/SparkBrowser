import assert from 'node:assert/strict';
import { createNativeKeySender } from './native-keyboard.mjs';
import { createNativePointerSender } from './native-pointer.mjs';

export async function checkCaretBrowsing(context) {
  const { invoke, until, wait, fixture, withWebview, request, sessionId, click, script } = context;
  const keys = async actions => request('POST', `/session/${sessionId()}/actions`, {
    actions: [{ type: 'key', id: 'caret-browsing-keys', actions }],
  });
  const nativeKey = process.platform === 'win32' ? createNativeKeySender(context.directory) : null;
  let enabling = true;
  const nativePointer = process.platform === 'win32' ? createNativePointerSender(context.directory) : null;
  if (nativePointer) context.ownCleanup(() => nativePointer([{ type: 'pointerUp', button: 0 }]));
  let contentOrigin;
  const pageClick = async selector => {
    if (!nativePointer) return click(selector);
    await context.focusWindow();
    const point = await script('const r=document.querySelector(arguments[0]).getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2}', selector);
    await nativePointer([{type:'pointerMove',duration:0,x:contentOrigin.x+point.x,y:contentOrigin.y+point.y},{type:'pointerDown',button:0},{type:'pointerUp',button:0}]);
  };
  const key = async value => {
    if (nativeKey && value === '\uE037') {
      // F7 belongs to the host's WebView2 accelerator, outside DOM key dispatch.
      await nativeKey.press(0x76);
      if (enabling) {
        // WebView2's first-use caret confirmation accepts Enter. This is a
        // host prompt, outside the page DOM and its synthetic keyboard events.
        await nativeKey.press(0x0D, true);
      }
      enabling = !enabling;
    } else await keys([{ type: 'keyDown', value }, { type: 'keyUp', value }]);
  };
  const selectNextCharacter = () => keys([
    { type: 'keyDown', value: '\uE008' }, { type: 'keyDown', value: '\uE014' },
    { type: 'keyUp', value: '\uE014' }, { type: 'keyUp', value: '\uE008' },
  ]);
  const selectedText = () => script('return window.getSelection()?.toString() ?? ""');

  const firstUrl = fixture.origin + '/caret-browsing?tab=first';
  await context.focusWindow();
  const first = (await invoke('create_tab', { url: firstUrl })).tabs.find(tab => tab.is_active);
  contentOrigin = await script('const r=document.querySelector(".viewport-frame").getBoundingClientRect();return {x:r.x,y:r.y}');

  await withWebview('/caret-browsing?tab=first', async () => {
    await pageClick('#text');
    assert.equal(await script('return document.hasFocus()'), true, 'native page owns focus before the host accelerator');
    await selectNextCharacter();
    assert.equal(await selectedText(), '', 'caret browsing starts disabled for a new application session');

    await key('\uE037'); // F7
    await wait(150);
    await pageClick('#text');
    await selectNextCharacter();
    await until(async () => (await selectedText()).length > 0, 'native caret selects page text after F7');

    await click('#link');
    await selectNextCharacter();
    assert.ok((await selectedText()).length > 0, 'link text can be selected without activating it');

    const frame = await request('POST', `/session/${sessionId()}/element`, { using: 'css selector', value: '#frame' });
    await request('POST', `/session/${sessionId()}/frame`, { id: frame });
    try {
      await click('#frame-text');
      await selectNextCharacter();
      assert.ok((await selectedText()).length > 0, 'keyboard selection works inside an embedded frame');
    } finally { await request('POST', `/session/${sessionId()}/frame`, { id: null }); }
  });

  const secondUrl = fixture.origin + '/caret-browsing?tab=second';
  const second = (await invoke('create_tab', { url: secondUrl })).tabs.find(tab => tab.is_active);
  const workspaceId = `caret-browsing-${Date.now()}`;
  await invoke('create_workspace', { id: workspaceId, name: workspaceId, icon: 'C', accentPrimary: '#4c83d1', accentSecondary: '#8493a8' });
  await invoke('move_tab_to_workspace', { tabId: second.id, workspaceId });
  await withWebview('/caret-browsing?tab=second', async () => {
    await pageClick('#text');
    await selectNextCharacter();
    await until(async () => (await selectedText()).length > 0, 'enabled session state reaches newly created tabs');

    await key('\uE037'); // Toggle off in a different tab.
    await wait(150);
    await script('window.getSelection()?.removeAllRanges()');
    await pageClick('#text');
    await selectNextCharacter();
    assert.equal(await selectedText(), '', 'F7 disables caret selection in the current tab');
    await click('#field');
    await selectNextCharacter();
    assert.equal(await script('return document.querySelector("#field").value'), 'Editable input remains usable', 'F7 does not alter editable field content');
    assert.ok(await script('const field=document.querySelector("#field");return field.selectionStart < field.selectionEnd'), 'editable fields retain normal keyboard selection');
  });

  await invoke('switch_tab', { tabId: first.id });
  await withWebview('/caret-browsing?tab=first', async () => {
    await script('window.getSelection()?.removeAllRanges()');
    await pageClick('#text');
    await selectNextCharacter();
    await until(async () => (await selectedText()) === '', 'F7 state is shared across tabs and workspaces');
  });
}
