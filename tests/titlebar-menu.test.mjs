import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

function load(path, imports) {
  const source = ts.transpileModule(readFileSync(path, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, { exports, require: name => {
    if (!(name in imports)) throw Error('Missing import: ' + name);
    return imports[name];
  } });
  return exports;
}

function fixture(invoke = async () => {}) {
  const writable = initial => {
    let value = initial;
    return { set: next => { value = next; }, subscribe: listener => { listener(value); return () => {}; } };
  };
  const get = store => { let value; store.subscribe(next => { value = next; })(); return value; };
  const calls = [];
  const overlays = load('src/lib/services/bridge/overlays.ts', {
    'svelte/store': { get }, '../../i18n': {}, '@tauri-apps/api/event': {},
    './runtime': { isTauriRuntime: () => true, invoke: async (command, args) => {
      calls.push(args.visible);
      await invoke(command, args);
    } },
  });
  const store = load('src/lib/stores/titlebarMenuStore.ts', {
    'svelte/store': { get, writable }, './workspaceSelectionStore': { activeWorkspaceId: writable('ws') },
    '../services/bridge/overlays': overlays,
  });
  const anchor = { isConnected: true, getClientRects: () => [1], focus: () => { anchor.focused++; }, focused: 0 };
  return { ...store, overlays, anchor, calls, current: () => get(store.titlebarMenu) };
}

test('dismissing a pending menu prevents a delayed native response from reopening it', async () => {
  let release;
  const pending = new Promise(resolve => { release = resolve; });
  const f = fixture(async (_, args) => { if (args.visible) await pending; });
  const opening = f.openTitlebarMenu(f.anchor);
  await Promise.resolve();
  const closing = f.closeTitlebarMenu();
  release();
  await Promise.all([opening, closing]);
  assert.equal(f.current(), null);
  assert.deepEqual(f.calls, [true, false]);
});

test('menu-to-settings handoff restores pages for the native settings surface', async () => {
  const f = fixture();
  f.overlays.onOverlayOpening(id => { if (id !== 'titlebar-menu') void f.closeTitlebarMenu(); });
  await f.openTitlebarMenu(f.anchor);
  assert.ok(f.current());
  await f.overlays.setOverlayVisibility(true, 'settings');
  assert.equal(f.current(), null);
  assert.equal(f.calls.at(-1), false);
  await f.overlays.setOverlayVisibility(false, 'settings');
  assert.equal(f.calls.at(-1), false);
});

test('failed menu presentation releases the overlay and allows retry with focus restoration', async () => {
  let fail = true;
  const f = fixture(async (_, args) => { if (args.visible && fail) { fail = false; throw Error('injected failure'); } });
  await assert.rejects(f.openTitlebarMenu(f.anchor), /injected failure/);
  assert.equal(f.current(), null);
  assert.deepEqual(f.calls, [true, false]);
  await f.openTitlebarMenu(f.anchor);
  await f.closeTitlebarMenu(true);
  assert.equal(f.current(), null);
  assert.equal(f.anchor.focused, 1);
});
