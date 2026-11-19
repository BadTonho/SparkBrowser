import assert from 'node:assert/strict';

// The `keyboard` scenario only runs on Windows, so this covers the shortcuts a web page sends to the
// browser on Linux. The page bridge reports them to Rust, which used to drop every one of them
// because the window was never found when checking its focus.
export async function checkKeyboardLinux(context) {
  if (process.platform !== 'linux') return;
  const { invoke, until, fixture, script, request, sessionId, withWebview } = context;
  const tab = (await invoke('create_tab', { url: fixture.origin + '/page/keyboard-linux' })).tabs.find(t => t.is_active);
  await until(() => fixture.reports.has('keyboard-linux'), 'keyboard fixture page');
  await context.focusWindow();
  // Keys are sent to the page's own document, like a user pressing them while reading it.
  const press = (...keys) => withWebview('/page/keyboard-linux', () => request('POST', `/session/${sessionId()}/actions`, {
    actions: [{ type: 'key', id: 'keyboard-linux', actions: [
      { type: 'keyDown', value: '' },
      ...keys.flatMap(key => [{ type: 'keyDown', value: key }, { type: 'keyUp', value: key }]),
      { type: 'keyUp', value: '' },
    ] }],
  }));
  const tabCount = async () => (await invoke('get_tabs')).tabs.length;

  // Ctrl+L focuses the address bar of the shell.
  await press('l');
  await until(() => script('return document.activeElement?.matches(".omnibox-input")'), 'Ctrl+L from a page focuses the address bar');

  // Ctrl+T opens a new tab and makes it the active one.
  const before = await tabCount();
  await press('t');
  await until(async () => await tabCount() === before + 1, 'Ctrl+T from a page opens a new tab');
  assert.notEqual((await invoke('get_tabs')).tabs.find(t => t.is_active)?.id, tab.id, 'the new tab is active');
}
