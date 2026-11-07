import assert from 'node:assert/strict';

export async function checkCaretBrowsing(context) {
  const { invoke, until, wait, fixture, withWebview, request, sessionId, click, script } = context;
  const keys = async actions => request('POST', `/session/${sessionId()}/actions`, {
    actions: [{ type: 'key', id: 'caret-browsing-keys', actions }],
  });
  const key = value => keys([{ type: 'keyDown', value }, { type: 'keyUp', value }]);
  const selectNextCharacter = () => keys([
    { type: 'keyDown', value: '\uE008' }, { type: 'keyDown', value: '\uE014' },
    { type: 'keyUp', value: '\uE014' }, { type: 'keyUp', value: '\uE008' },
  ]);
  const selectedText = () => script('return window.getSelection()?.toString() ?? ""');

  const firstUrl = fixture.origin + '/caret-browsing?tab=first';
  const first = (await invoke('create_tab', { url: firstUrl })).tabs.find(tab => tab.is_active);

  await withWebview('/caret-browsing?tab=first', async () => {
    await click('#text');
    await selectNextCharacter();
    assert.equal(await selectedText(), '', 'caret browsing starts disabled for a new application session');

    await key('\uE037'); // F7
    await wait(150);
    await click('#text');
    await selectNextCharacter();
    await until(async () => (await selectedText()).length > 0, 'native caret selects page text after F7');

    await click('#link');
    await selectNextCharacter();
    assert.ok((await selectedText()).length > 0, 'link text can be selected without activating it');

    await script('document.querySelector("#frame").contentWindow.focus()');
    await selectNextCharacter();
    assert.ok(await script('return document.querySelector("#frame").contentDocument.getSelection().toString().length > 0'), 'keyboard selection works inside an embedded frame');
  });

  const secondUrl = fixture.origin + '/caret-browsing?tab=second';
  const second = (await invoke('create_tab', { url: secondUrl })).tabs.find(tab => tab.is_active);
  const workspaceId = `caret-browsing-${Date.now()}`;
  await invoke('create_workspace', { id: workspaceId, name: workspaceId, icon: 'C', accentPrimary: '#4c83d1', accentSecondary: '#8493a8' });
  await invoke('move_tab_to_workspace', { tabId: second.id, workspaceId });
  await withWebview('/caret-browsing?tab=second', async () => {
    await click('#text');
    await selectNextCharacter();
    await until(async () => (await selectedText()).length > 0, 'enabled session state reaches newly created tabs');

    await key('\uE037'); // Toggle off in a different tab.
    await wait(150);
    await click('#field');
    await selectNextCharacter();
    assert.equal(await script('return document.querySelector("#field").value'), 'Editable input remains usable', 'F7 does not alter editable field content');
    assert.ok(await script('const field=document.querySelector("#field");return field.selectionStart < field.selectionEnd'), 'editable fields retain normal keyboard selection');
  });

  await invoke('switch_tab', { tabId: first.id });
  await withWebview('/caret-browsing?tab=first', async () => {
    await click('#text');
    await selectNextCharacter();
    await until(async () => (await selectedText()) === '', 'F7 state is shared across tabs and workspaces');
  });
}
