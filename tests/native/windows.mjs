import assert from 'node:assert/strict';

// A second browser window has its own shell, tabs and selection; the profile is shared.
export async function checkBrowserWindows({ invoke, until, fixture, request, sessionId, wait }) {
  const base = () => `/session/${sessionId()}`;
  const handles = () => request('GET', `${base()}/window/handles`);
  const before = await handles();
  const firstTab = (await invoke('get_tabs')).tabs.find(tab => tab.is_active).id;
  const id = await invoke('open_browser_window');
  assert.equal(id, 'window-2');
  const previous = await request('GET', `${base()}/window`);
  // The new window adds a shell and a page; only the shell holds the browser UI.
  const second = await until(async () => {
    for (const handle of (await handles()).filter(handle => !before.includes(handle))) {
      try {
        await request('POST', `${base()}/window`, { handle });
        const result = await request('POST', `${base()}/execute/sync`, { script: 'return Boolean(window.__TAURI_INTERNALS__) && Boolean(document.querySelector(".browser-root"))', args: [] });
        if (result) return handle;
      } catch { /* the page may still be loading */ }
      finally { await request('POST', `${base()}/window`, { handle: previous }); }
    }
    return null;
  }, 'second shell appears');
  // Runs a command from the shell of the second window.
  const invokeIn = async (command, args = {}) => {
    await request('POST', `${base()}/window`, { handle: second });
    try {
      const result = await request('POST', `${base()}/execute/async`, {
        script: 'const done = arguments[arguments.length - 1]; window.__TAURI_INTERNALS__.invoke(arguments[0], arguments[1]).then(value => done({ value }), error => done({ error: String(error) }));',
        args: [command, args],
      });
      if (result.error) throw new Error(`${command}: ${result.error}`);
      return result.value;
    } finally { await request('POST', `${base()}/window`, { handle: previous }); }
  };
  let lastError = '';
  try { await until(async () => { try { return (await invokeIn('get_tabs')).tabs.length === 1; } catch (error) { lastError = String(error); return false; } }, 'second shell is ready'); } catch (error) { throw new Error(`${error.message}: ${lastError}`); }
  const other = await invokeIn('get_tabs');
  assert.equal(other.window_id, 'window-2');
  assert.notEqual(other.tabs[0].id, firstTab);
  assert.equal(other.tabs[0].window_id, 'window-2');
  assert.ok(other.tabs[0].is_active);

  // Tabs opened in one window never show in the other.
  const opened = (await invokeIn('create_tab', { url: fixture.origin + '/page/b' })).tabs;
  assert.equal(opened.length, 2);
  const main = await invoke('get_tabs');
  assert.equal(main.window_id, 'main');
  assert.ok(main.tabs.every(tab => tab.window_id === 'main'));
  assert.ok(main.tabs.find(tab => tab.id === firstTab)?.is_active, 'main keeps its own selection');
  assert.equal((await invoke('get_browser_windows')).length, 2);

  // Closing the window keeps its tabs reopenable and leaves main untouched.
  await invokeIn('plugin:window|close', { label: 'window-2' });
  await until(async () => (await invoke('get_browser_windows')).length === 1, 'second window closed');
  assert.ok((await invoke('get_closed_tabs')).length >= 2, 'its tabs can be reopened');
  assert.ok((await invoke('get_tabs')).tabs.some(tab => tab.id === firstTab));
  await wait(200);
}
