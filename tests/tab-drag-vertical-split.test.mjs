import { createSharedLifecycle } from "../src/lib/services/lifecycle.js";
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';
import { replaceGroupOrder } from '../src/lib/services/split-tab-groups.js';

function load(path, imports, globals = {}) {
  const source = ts.transpileModule(readFileSync(path, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, { exports, require: name => { if (name === "../services/lifecycle.js") return { createSharedLifecycle };
    if (!(name in imports)) throw Error('Missing test import: ' + name);
    return imports[name];
  }, ...globals });
  return exports;
}
const writable = initial => {
  let value = initial;
  return { set: next => { value = next; }, subscribe: listener => { listener(value); return () => {}; } };
};
const get = store => { let value; store.subscribe(next => { value = next; })(); return value; };

/** A vertical list in split view: a, b on the left, c on the right; `under` is what the pointer is over at release. */
function fixture(under, y) {
  const calls = [], listeners = new Map();
  const runtime = { isTauriRuntime: () => true, invoke: async (command, args) => {
    calls.push({ command, args });
    if (command === 'move_tab_in_split') return { tabs: [], split_layout: null };
  } };
  const bridge = load('src/lib/services/bridge/tab-drag.ts', { './runtime': runtime });
  const root = { addEventListener() {}, removeEventListener() {}, hasPointerCapture: () => false, setPointerCapture() {}, releasePointerCapture() {} };
  const layout = { workspace_id: 'ws', left_tab_id: 'a', right_tab_id: 'c', ratio: .5, focused_tab_id: 'a', left_tab_ids: ['a', 'b'], right_tab_ids: ['c'] };
  const store = load('src/lib/stores/tabDragStore.ts', {
    'svelte/store': { get, writable },
    './browserStore': {
      tabs: writable([{ id: 'a', workspace_id: 'ws', is_active: true }, { id: 'b', workspace_id: 'ws' }, { id: 'c', workspace_id: 'ws' }]),
      beginTabReorder() {}, previewTabReorder() {}, selectTab() {}, changeTabWorkspace() {},
      finishTabReorder: async () => {}, applyTabsSnapshot: () => { calls.push({ command: 'snapshot' }); },
    },
    './workspaceSelectionStore': { activeWorkspaceId: writable('ws') },
    './splitViewStore': { splitLayout: writable(layout), splitKey: () => 'k', splitGroupsKey: () => 'g' },
    './settingsStore': { tabLayout: writable('vertical') },
    './tabGroupsStore': { tabGroups: writable({ groups: [], members: {} }), addTabsToGroup() {}, removeTabsFromGroup() {} },
    './tabSelectionStore': { selectedTabIds: writable([]) },
    '../i18n': { locale: writable('en'), tr: key => key },
    '../services/ignored-error': { logIgnored() {} }, '../services/bridge/overlays': { onOverlayOpening: () => () => {} },
    '../services/bridge/tab-drag': bridge, '../services/bridge/windows': { detachTabToNewWindow: async () => {} },
    '../services/tab-drag-ghost': { createTabGhost: () => ({ move() {}, setHidden() {}, destroy() {} }) },
    '../services/split-tab-groups.js': { replaceGroupOrder },
    '../services/tab-drag-geometry.js': { crossedDragThreshold: () => true, splitDropSide: () => null },
    './mediaPopoverStore': { mediaPopover: writable(null), closeMedia: async () => {} },
    './privacyPopoverStore': { privacyPopover: writable(null), closePrivacy: async () => {} },
    './zoomPopoverStore': { zoomPopover: writable(null), closeZoom: async () => {} },
    '../services/bridge/runtime': runtime,
  }, {
    window: { addEventListener: (name, fn) => listeners.set(name, fn), removeEventListener: name => listeners.delete(name) },
    document: {
      body: { style: { cursor: '', userSelect: '' } },
      querySelector: selector => selector === '.browser-root' ? root : { getBoundingClientRect: () => ({ x: 300, y: 40, width: 800, height: 520, left: 300, right: 1100, top: 40, bottom: 560 }) },
      querySelectorAll: () => [], elementFromPoint: () => under,
    },
    CSS: { escape: value => value }, innerWidth: 1200, innerHeight: 700,
    requestAnimationFrame: () => 1, cancelAnimationFrame() {},
  });
  const event = (x, y) => ({ clientX: x, clientY: y, screenX: x, screenY: y, pointerId: 1, button: 0, isPrimary: true, target: { closest: () => null }, preventDefault() {} });
  store.startTabDrag(event(100, 100), 'b');
  listeners.get('pointermove')(event(100, y));
  return { calls, release: () => listeners.get('pointerup')(event(100, y)) };
}

/** The list as the real one lays out: separators at fixed heights; `y` is where the pointer is released. */
const list = (row) => {
  const separator = (side, top) => ({ dataset: side ? { paneSide: side } : {}, getBoundingClientRect: () => ({ top }) });
  const section = { querySelectorAll: () => [separator('right', 300), separator(undefined, 500)] };
  return { closest: selector => selector === '.tabs-section' ? section : selector === '.tabs-list' ? {} : selector.includes('.tab-row') ? row ?? null : null };
};

test('dropping a left tab on a right-side row moves it before that row', async () => {
  const row = { dataset: { tabId: 'c', paneSide: 'right' } };
  const f = fixture(list(row), 340);
  await f.release();
  const move = f.calls.find(c => c.command === 'move_tab_in_split');
  assert.ok(move, 'the move reached the backend');
  assert.deepEqual([move.args.tabId, move.args.side, move.args.beforeId], ['b', 'right', 'c']);
});

test('dropping a left tab anywhere in the right section, not only on its title, appends it there', async () => {
  const f = fixture(list(null), 420);
  await f.release();
  const move = f.calls.find(c => c.command === 'move_tab_in_split');
  assert.ok(move, 'the move reached the backend');
  assert.deepEqual([move.args.side, move.args.beforeId], ['right', null]);
});

test('dropping on its own section does not move it across the split', async () => {
  const f = fixture(list({ dataset: { tabId: 'a', paneSide: 'left' } }), 120);
  await f.release();
  assert.equal(f.calls.some(c => c.command === 'move_tab_in_split'), false);
});

test('dropping in the section of tabs outside the split does not move it', async () => {
  const f = fixture(list(null), 560);
  await f.release();
  assert.equal(f.calls.some(c => c.command === 'move_tab_in_split'), false);
});
