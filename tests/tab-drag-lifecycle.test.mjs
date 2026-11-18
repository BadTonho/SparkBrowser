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

function fixture({ failDrop = false } = {}) {
  const calls = [], listeners = new Map();
  let frame;
  let acknowledge;
  const pending = new Promise(resolve => { acknowledge = resolve; });
  const runtime = { isTauriRuntime: () => true, invoke: async (command, args) => {
    calls.push({ command, args });
    if (command === 'set_tab_drag_preview' && args.presentation) await pending;
    if (command === 'dock_tab_to_split') {
      if (failDrop) throw Error('injected docking failure');
      return { tabs: [], split_layout: { left_tab_id: 'a', right_tab_id: 'b' } };
    }
  } };
  const bridge = load('src/lib/services/bridge/tab-drag.ts', { './runtime': runtime });
  let captured = false;
  const root = {
    addEventListener() {}, removeEventListener() {}, hasPointerCapture: () => captured,
    setPointerCapture: () => { captured = true; }, releasePointerCapture: () => { captured = false; },
  };
  const items = writable([
    { id: 'a', workspace_id: 'ws', is_active: true },
    { id: 'b', workspace_id: 'ws', is_active: false },
  ]);
  const browser = {
    tabs: items, beginTabReorder() {}, previewTabReorder() {}, selectTab() {}, changeTabWorkspace() {},
    finishTabReorder: async cancel => { calls.push({ command: 'finish-reorder', cancel }); },
    applyTabsSnapshot: snapshot => { calls.push({ command: 'snapshot', snapshot }); },
  };
  const store = load('src/lib/stores/tabDragStore.ts', {
    'svelte/store': { get, writable }, './browserStore': browser,
    './workspaceSelectionStore': { activeWorkspaceId: writable('ws') },
    './splitViewStore': { splitLayout: writable(null), splitKey: () => null },
    './settingsStore': { tabLayout: writable('horizontal') },
    '../i18n': { locale: writable('en'), tr: key => key },
    '../services/ignored-error': { logIgnored() {} }, '../services/bridge/overlays': { onOverlayOpening: () => () => {} },
    '../services/bridge/tab-drag': bridge,
    '../services/tab-drag-ghost': { createTabGhost: () => ({ move() {}, setHidden() {}, destroy() {} }) },
    '../services/split-tab-groups.js': { replaceGroupOrder },
    '../services/tab-drag-geometry.js': { crossedDragThreshold: () => true, splitDropSide: () => 'right' },
    './mediaPopoverStore': { mediaPopover: writable(null), closeMedia: async () => {} },
    './privacyPopoverStore': { privacyPopover: writable(null), closePrivacy: async () => {} },
    './zoomPopoverStore': { zoomPopover: writable(null), closeZoom: async () => {} },
    '../services/bridge/runtime': runtime,
  }, {
    window: {
      addEventListener: (name, fn) => listeners.set(name, fn),
      removeEventListener: name => listeners.delete(name),
    },
    document: {
      body: { style: { cursor: '', userSelect: '' } },
      querySelector: selector => selector === '.browser-root' ? root : {
        getBoundingClientRect: () => ({ x: 0, y: 80, width: 800, height: 520 }),
      },
      querySelectorAll: () => [], elementFromPoint: () => ({ closest: () => null }),
    },
    CSS: { escape: value => value }, innerWidth: 800, innerHeight: 600,
    requestAnimationFrame: callback => { frame = callback; return 1; }, cancelAnimationFrame() { frame = undefined; },
  });
  const event = (x, y) => ({ clientX: x, clientY: y, pointerId: 1, button: 0, isPrimary: true,
    target: { closest: () => null }, preventDefault() {} });
  store.startTabDrag(event(100, 20), 'b');
  listeners.get('pointermove')(event(790, 300));
  return { calls, acknowledge, bridge, store, captured: () => captured,
    move: (x, y) => listeners.get('pointermove')(event(x, y)), frame: () => frame?.(),
    release: () => listeners.get('pointerup')(event(790, 300)) };
}

test('pointer movements are coalesced until the next frame and release flushes the latest position', async () => {
  const f = fixture();
  f.move(700, 300); f.move(750, 300);
  assert.equal(f.calls.filter(c => c.command === 'set_tab_drag_preview').length, 0);
  f.frame();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(f.calls.filter(c => c.command === 'set_tab_drag_preview').length, 1);
  const release = f.release();
  f.acknowledge();
  await release;
  const previews = f.calls.filter(c => c.command === 'set_tab_drag_preview' && c.args.presentation);
  assert.equal(previews.at(-1).args.presentation.pointer.x, 790);
});

test('drop drains the pending preview and commits before removing its geometry', async () => {
  const f = fixture();
  await Promise.resolve();
  const release = f.release();
  assert.equal(f.captured(), false, 'DOM capture ends immediately');
  assert.equal(f.calls.some(c => c.command === 'dock_tab_to_split'), false);
  assert.equal(f.calls.some(c => c.command === 'set_tab_drag_preview' && !c.args.presentation), false);
  f.acknowledge();
  await release;
  const commands = f.calls.map(c => c.command);
  assert.ok(commands.indexOf('dock_tab_to_split') > commands.indexOf('finish-reorder'));
  assert.equal(commands.at(-1), 'set_tab_drag_preview');
  assert.equal(f.calls.at(-1).args.presentation, null);
  assert.ok(commands.indexOf('snapshot') < commands.length - 1);
});

test('a rejected docking commit still clears the preview and restores reorder', async () => {
  const f = fixture({ failDrop: true });
  const release = f.release();
  f.acknowledge();
  await assert.rejects(release, /injected docking failure/);
  assert.equal(f.calls.at(-1).args.presentation, null);
  assert.equal(f.calls.some(c => c.command === 'snapshot'), false);
  assert.ok(f.calls.filter(c => c.command === 'finish-reorder').every(c => c.cancel));
});

test('cancelling with a pending update clears it without confirming the split', async () => {
  const f = fixture();
  await Promise.resolve();
  f.store.cancelTabDrag();
  f.acknowledge();
  await f.bridge.flushDragPreview();
  assert.equal(f.captured(), false);
  assert.equal(f.calls.at(-1).args.presentation, null);
  assert.equal(f.calls.some(c => c.command === 'dock_tab_to_split'), false);
});
