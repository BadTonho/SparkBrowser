import { createSharedLifecycle } from "../src/lib/services/lifecycle.js";
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';
function load(path, imports = {}, globals = {}) {
  const exports = {};
  runInNewContext(ts.transpileModule(readFileSync(path, 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText,
    { exports, require: name => { if (name === '../services/lifecycle.js') return { createSharedLifecycle }; if (name === './download-visual') return load('src/lib/services/download-visual.ts', {}, { atob }); if (!(name in imports)) throw Error(name); return imports[name]; }, ...globals });
  return exports;
}
const { summarizeDownloads } = load('src/lib/services/download-progress.ts');
const { createDownloadNotices } = load('src/lib/services/download-notices.ts');
const task = (status, received = 0, total = null) => ({ status, received, total });
test('toolbar combines transfer bytes, excludes other states and clamps reported values', () => {
  const progress = summarizeDownloads([task('Downloading', 10, 100), task('Downloading', 900, 1000), task('Paused', 0, 100000), task('Completed', 999999, 999999)]);
  assert.equal(progress.kind, 'determinate'); assert.equal(progress.count, 2);
  assert.equal(progress.percent, 910 / 1100 * 100);
  assert.equal(summarizeDownloads([task('Downloading', 999, 100)]).percent, 100);
  assert.equal(summarizeDownloads([task('Downloading', -10, 100)]).percent, 0);
});
test('unknown sizes and processing never invent a percentage; only pauses remain static', () => {
  for (const total of [null, 0, -1, NaN, Infinity]) assert.equal(summarizeDownloads([task('Downloading', 10, total)]).kind, 'indeterminate');
  for (const status of ['Queued', 'Verifying', 'Finalizing']) assert.equal(summarizeDownloads([task(status)]).percent, null);
  const paused = summarizeDownloads([task('Paused', 25, 100), task('Pausing', 75, 100)]);
  assert.equal(paused.kind, 'paused'); assert.equal(paused.percent, 50);
  assert.equal(summarizeDownloads([task('Paused', 0, null)]).percent, null);
  for (const status of ['Completed', 'Cancelled', 'Failed', 'Cancelling', 'Interrupted', 'RestartRequired', 'VerificationFailed']) assert.equal(summarizeDownloads([task(status)]).kind, 'idle');
  assert.equal(summarizeDownloads([task('Paused', 50, 100), task('Downloading', 20, 100)]).percent, 20);
});
test('downloads panel uses an 8px gap and margins, 480px width and at most 560px height', () => {
  const { panelPosition } = load('src/lib/services/panel-presentation.ts');
  for (const width of [320, 680, 1280]) for (const left of [8, width / 2, width - 36]) {
    const position = panelPosition({ left, top: 40, width: 28, height: 28 }, width, 900, { width: 480, maxHeight: 560 });
    assert.equal(position.top, 76); assert.equal(position.width, Math.min(480, width - 16));
    assert.ok(position.left >= 8 && position.left + position.width <= width - 8); assert.equal(position.maxHeight, 560);
  }
  assert.equal(panelPosition({ left: 600, top: 40, width: 28, height: 28 }, 680, 480, { width: 480, maxHeight: 560 }).maxHeight, 396);
});

function fixture(tick = async () => {}, settle = async () => {}) {
  const writable = value => ({ set(next) { value = next; }, update(fn) { value = fn(value); }, subscribe(fn) { fn(value); return () => {}; } });
  const get = store => { let result; store.subscribe(value => result = value)(); return result; };
  const derived = (stores, fn) => ({ subscribe(callback) { callback(Array.isArray(stores) ? fn(stores.map(get)) : fn(get(stores))); return () => {}; } });
  const anchor = () => ({ isConnected: true, visible: true, focused: 0, getClientRects() { return this.visible ? [1] : []; }, focus() { this.focused++; } });
  const button = anchor(), more = anchor(), zen = writable(true), calls = [], notices = [];
  const noticesModule = load('src/lib/stores/downloadNoticesStore.ts', {
    'svelte/store': { writable }, '../services/storage.js': { safeStorage: { getItem: () => null, setItem: () => true } },
    '../services/download-notices': { createDownloadNotices },
  });
  let onTask, resolveTasks, loads = 0, showWhenDone = false;
  const api = load('src/lib/stores/downloadsStore.ts', {
    'svelte/store': { get, writable, derived }, '../services/ignored-error': { logIgnored() {} }, svelte: { tick }, '../i18n': { t: writable(key => key) },
    '../services/download-progress': { summarizeDownloads }, '../services/user-errors.js': { describeError: String },
    './downloadNoticesStore': noticesModule, './downloadSettingsStore': { isShowingDownloadsWhenDone: () => showWhenDone },
    '../services/bridge/overlays': { notifyOpening: id => notices.push(id), settleOverlayVisibility: settle }, '../services/bridge/downloads-popover': { setDownloadsPopover: async (...args) => calls.push(args) },
    './settingsStore': { isZenMode: zen }, '../services/tauri-bridge': {
      fetchDownloads: async () => [], fetchDownloadTasks: () => { loads++; return new Promise(resolve => resolveTasks = resolve); },
      onDownloadTaskUpdated: async callback => { onTask = callback; return () => {}; }, onDownloadsUpdated: async () => () => {}, isTauriRuntime: () => true,
    },
  }, { document: { hasFocus: () => true, querySelector: selector => selector.includes('titlebar-actions') ? button : more } });
  return { ...api, showWhenDone: value => { showWhenDone = value; }, get, button, more, zen, calls, notices, emit: value => onTask(value), resolve: value => resolveTasks(value), loads: () => loads };
}
test('startup loads progress without opening the panel and a late snapshot preserves newer events', async () => {
  const f = fixture(), stop = f.initializeDownloadEvents(); await new Promise(setImmediate);
  assert.equal(f.loads(), 1); assert.equal(f.get(f.isDownloadsDrawerOpen), false);
  f.emit({ id: 'one', revision: 2, ...task('Downloading', 50, 100) });
  f.resolve([{ id: 'one', revision: 1, ...task('Downloading', 20, 100) }]); await new Promise(setImmediate);
  assert.equal(f.get(f.downloadProgress).percent, 50);
  stop(); f.emit({ id: 'one', revision: 3, ...task('Completed', 100, 100) });
  assert.equal(f.get(f.downloadProgress).percent, 50, 'disposed listeners cannot mutate progress');
});
test('opening restores the toolbar, resolves More, toggles the trigger and cancels pending openings', async () => {
  const f = fixture(); f.button.visible = false; await f.openDownloadsDrawer();
  assert.equal(f.get(f.zen), false); assert.equal(f.get(f.downloadsSession).anchor, f.more);
  assert.deepEqual(f.notices, ['downloads']); await f.openDownloadsDrawer(f.more);
  assert.equal(f.get(f.isDownloadsDrawerOpen), false); assert.equal(f.more.focused, 1);
  let resolve; const pending = fixture(() => new Promise(r => resolve = r));
  const opening = pending.openDownloadsDrawer(); await pending.closeDownloadsDrawer(); resolve(); await opening;
  assert.equal(pending.get(pending.isDownloadsDrawerOpen), false); assert.equal(pending.calls.length, 0);
});
test('native presentation waits for a modal dismissal and cancelling that wait never reopens it', async () => {
  let release;
  const f = fixture(async () => {}, () => new Promise(resolve => release = resolve));
  const opening = f.openDownloadsDrawer(); await new Promise(setImmediate);
  assert.equal(f.get(f.isDownloadsDrawerOpen), false);
  release(); await opening; assert.equal(f.get(f.isDownloadsDrawerOpen), true);
  await f.closeDownloadsDrawer();
  const cancelled = f.openDownloadsDrawer(); await new Promise(setImmediate);
  await f.closeDownloadsDrawer(); release(); await cancelled;
  assert.equal(f.get(f.isDownloadsDrawerOpen), false);
});

test('new results remain visible after keyboard/header dismissal and opening acknowledges them', async () => {
  const f = fixture(); f.initializeDownloadEvents(); await new Promise(setImmediate);
  f.resolve([]); await new Promise(setImmediate);
  await f.openDownloadsDrawer();
  f.emit({ id: 'one', revision: 1, ...task('Completed', 100, 100) });
  assert.equal(f.get(f.downloadProgress).kind, 'completed');
  await f.closeDownloadsDrawer(true);
  assert.equal(f.get(f.downloadProgress).kind, 'completed');
  await f.openDownloadsDrawer();
  assert.equal(f.get(f.downloadProgress).kind, 'idle');
  f.emit({ id: 'two', revision: 1, ...task('Failed') });
  assert.equal(f.get(f.downloadProgress).kind, 'error');
  await f.openDownloadsDrawer(f.button);
  assert.equal(f.get(f.downloadProgress).kind, 'idle');
  assert.equal(f.get(f.isDownloadsDrawerOpen), false);
});

test('error notices take priority, then transfers, processing, pauses and completion notices', () => {
  const notices = { error: 1, completed: 1 };
  assert.equal(summarizeDownloads([task('Downloading', 20, 100)], notices).kind, 'error');
  notices.error = 0;
  assert.equal(summarizeDownloads([task('Downloading', 20, 100)], notices).kind, 'determinate');
  assert.equal(summarizeDownloads([task('Verifying'), task('Paused')], notices).kind, 'indeterminate');
  assert.equal(summarizeDownloads([task('Paused', 20, 100)], notices).kind, 'paused');
  assert.equal(summarizeDownloads([], notices).kind, 'completed');
  assert.equal(summarizeDownloads([], notices).percent, 100);
});

test('trigger hit testing distinguishes the button from an ordinary outside dismissal', () => {
  const { isPanelTrigger } = load('src/lib/services/panel-presentation.ts');
  const anchor = { left: 100, top: 30, width: 28, height: 28 };
  assert.equal(isPanelTrigger(anchor, 114, 44), true);
  for (const [x, y] of [[99, 44], [129, 44], [114, 29], [114, 59], [NaN, 44]]) assert.equal(isPanelTrigger(anchor, x, y), false);
});

test('a download that finishes opens the panel only when asked to, and only if it was seen running', async () => {
  const f = fixture(); f.initializeDownloadEvents(); await new Promise(setImmediate);
  f.resolve([]); await new Promise(setImmediate);
  const settle = async () => { for (let i = 0; i < 6; i++) await new Promise(setImmediate); };
  f.showWhenDone(true);
  // Seen for the first time already completed (an old one coming back): nothing opens.
  f.emit({ id: 'old', revision: 1, ...task('Completed', 100, 100) }); await settle();
  assert.equal(f.get(f.isDownloadsDrawerOpen), false);
  // Seen running, then completed: the panel opens.
  f.emit({ id: 'new', revision: 1, ...task('Downloading', 10, 100) }); await settle();
  assert.equal(f.get(f.isDownloadsDrawerOpen), false, 'a running download never opens it');
  f.emit({ id: 'new', revision: 2, ...task('Completed', 100, 100) }); await settle();
  assert.equal(f.get(f.isDownloadsDrawerOpen), true);
  await f.closeDownloadsDrawer();
  // Turned off: it stays closed.
  f.showWhenDone(false);
  f.emit({ id: 'quiet', revision: 1, ...task('Downloading', 10, 100) });
  f.emit({ id: 'quiet', revision: 2, ...task('Completed', 100, 100) }); await settle();
  assert.equal(f.get(f.isDownloadsDrawerOpen), false);
});
