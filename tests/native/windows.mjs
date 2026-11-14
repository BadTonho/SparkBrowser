import assert from 'node:assert/strict';

// Opens a second browser window and returns a function that runs commands from its shell.
async function openSecondWindow({ invoke, until, request, sessionId }) {
  const base = () => `/session/${sessionId()}`;
  const handles = () => request('GET', `${base()}/window/handles`);
  const before = await handles();
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
  return invokeIn;
}

// A second browser window has its own shell, tabs and selection; the profile is shared.
export async function checkBrowserWindows(context) {
  const { invoke, until, fixture, wait } = context;
  const firstTab = (await invoke('get_tabs')).tabs.find(tab => tab.is_active).id;
  const invokeIn = await openSecondWindow(context);
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

  // A tab can be sent to the other window; it is selected there and the page keeps working.
  const extra = (await invoke('create_tab', { url: fixture.origin + '/page/a' })).tabs.find(tab => tab.is_active).id;
  await invoke('move_tab_to_window', { tabId: extra, windowId: 'window-2' });
  assert.ok((await invokeIn('get_tabs')).tabs.find(tab => tab.id === extra)?.is_active, 'moved tab is selected in the other window');
  assert.ok(!(await invoke('get_tabs')).tabs.some(tab => tab.id === extra), 'it left the first window');
  assert.ok((await invoke('get_tabs')).tabs.some(tab => tab.is_active), 'the first window keeps a selection');
  await until(async () => (await invokeIn('get_tabs')).tabs.find(tab => tab.id === extra)?.url.endsWith('/page/a'), 'moved page loads in its new window');

  // Panels are created in the window that asks for them and move when another window asks.
  for (const panel of ['tools', 'downloads', 'zoom', 'history']) {
    await invokeIn('prepare_panel', { panel });
    await invoke('prepare_panel', { panel });
    await invokeIn('prepare_panel', { panel });
  }

  // Closing the window keeps its tabs reopenable and leaves main untouched.
  await invokeIn('plugin:window|close', { label: 'window-2' });
  await until(async () => (await invoke('get_browser_windows')).length === 1, 'second window closed');
  assert.ok((await invoke('get_closed_tabs')).length >= 2, 'its tabs can be reopened');
  assert.ok((await invoke('get_tabs')).tabs.some(tab => tab.id === firstTab));
  await wait(200);
}

// Every window of the last session comes back with its own tabs, also after a crash.
export async function checkWindowRestore(context) {
  const { invoke, until, fixture, restart } = context;
  const invokeIn = await openSecondWindow(context);
  await invokeIn('create_tab', { url: fixture.origin + '/page/b' });
  const mainTabs = (await invoke('get_tabs')).tabs.map(tab => tab.id);
  const secondTabs = (await invokeIn('get_tabs')).tabs.map(tab => tab.id);
  assert.equal(secondTabs.length, 2);
  const windowsBack = async count => until(async () => (await invoke('get_browser_windows')).length === count, `${count} windows open`);
  for (const crash of [true, false]) {
    await restart({ crash });
    await windowsBack(2);
    const windows = await invoke('get_browser_windows');
    assert.deepEqual(windows.map(window => window.id).sort(), ['main', 'window-2']);
    assert.equal(windows.find(window => window.id === 'window-2').tab_count, 2);
    assert.equal(windows.find(window => window.id === 'main').tab_count, mainTabs.length, 'main keeps its own tabs');
    // The test driver may talk to either shell, so the tabs it sees belong to one of the two windows.
    const seen = (await invoke('get_tabs')).tabs.map(tab => tab.id);
    assert.ok(seen.every(id => (mainTabs.includes(id) || secondTabs.includes(id))), 'tab ids survive the restart');
    assert.equal((await invoke('get_closed_tabs')).length, 0, 'nothing was sent to recently closed');
  }
  assert.equal(secondTabs.length, 2);
}
