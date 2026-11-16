import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { dirname, resolve } from 'node:path';
import ts from 'typescript';
import * as stores from 'svelte/store';
import { createSnapshotGate } from '../src/lib/services/tab-snapshots.js';
import { mergeSessionObservations, mergeNavigationObservation } from '../src/lib/services/session-reconciliation.js';

function domain(bridge = {}) {
  const modules = new Map();
  const mocks = {
    'svelte/store': stores,
    'svelte/motion': { spring: stores.writable },
    [resolve('src/lib/services/new-tab')]: { NEW_TAB_URL: 'about:newtab', isNewTab: url => url === 'about:newtab' },
    [resolve('src/lib/i18n')]: { tr: key => key },
    [resolve('src/lib/services/tab-snapshots.js')]: { createSnapshotGate },
    [resolve('src/lib/services/session-reconciliation.js')]: { mergeSessionObservations, mergeNavigationObservation },
    [resolve('src/lib/stores/searchEngineStore')]: { defaultSearchEngine: stores.writable('google') },
    [resolve('src/lib/services/tauri-bridge')]: { isTauriRuntime: () => true, currentWindowId: () => 'main', ...bridge },
    [resolve('src/lib/services/split-tab-groups.js')]: {},
  };
  function load(path) {
    if (modules.has(path)) return modules.get(path);
    const exports = {};
    modules.set(path, exports);
    const code = ts.transpileModule(readFileSync(path, 'utf8'), {
      compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
    }).outputText;
    runInNewContext(code, { exports, require(name) {
      const target = name.startsWith('.') ? resolve(dirname(path), name) : name;
      if (target in mocks) return mocks[target];
      if (target.startsWith(resolve('src/lib/stores'))) return load(target + '.ts');
      throw new Error(`Unexpected domain dependency: ${name}`);
    } });
    return exports;
  }
  return load(resolve('src/lib/stores/browser/actions.ts'));
}
const tab = (id, active = false) => ({ id, url: 'https://example.test/' + id, title: id,
  workspace_id: 'ws-personal', is_active: active, is_hibernated: false, navigation: { revision: 0 } });
const packet = (revision, tabs) => ({ revision, tabs, split_layout: null });
const ids = store => Array.from(stores.get(store), item => item.id);

test('session facade is read-only and canceling drag cannot resurrect a closed tab', () => {
  const app = domain();
  assert.equal(app.tabs.set, undefined);
  assert.equal(app.currentUrl.set, undefined);
  app.applyTabsSnapshot(packet(1, [tab('a', true), tab('b'), tab('c')]));
  app.beginTabReorder();
  app.previewTabReorder(['c', 'b', 'a']);
  app.applyTabsSnapshot(packet(2, [tab('a', true), tab('c')]));
  app.finishTabReorder(true);
  assert.deepEqual(ids(app.tabs), ['a', 'c']);
});

test('failed reorder clears only its preview and preserves a newer session event', async () => {
  let reject;
  const pending = new Promise((_, fail) => { reject = fail; });
  const app = domain({ reorderTabs: () => pending });
  app.applyTabsSnapshot(packet(1, [tab('a', true), tab('b')]));
  const operation = app.reorderTabsList(['b', 'a']);
  await Promise.resolve();
  app.applyTabsSnapshot(packet(2, [tab('a', true), tab('c')]));
  reject(new Error('injected storage failure'));
  await assert.rejects(operation, /storage failure/);
  assert.deepEqual(ids(app.tabs), ['a', 'c']);
});

test('failed workspace move cannot overwrite unrelated navigation or tab events', async () => {
  let reject;
  const pending = new Promise((_, fail) => { reject = fail; });
  const app = domain({ moveTabToWorkspace: () => pending });
  app.applyTabsSnapshot(packet(1, [tab('a', true), tab('b')]));
  const operation = app.changeTabWorkspace('a', 'ws-work');
  app.applyTabsSnapshot(packet(2, [tab('a', true), tab('c')]));
  app.applyNavigationPatch({ id: 'a', url: 'https://example.test/latest', title: 'latest', navigation: { revision: 3 } });
  reject(new Error('injected storage failure'));
  await assert.rejects(operation, /storage failure/);
  assert.deepEqual(ids(app.tabs), ['a', 'c']);
  assert.equal(stores.get(app.tabs)[0].url, 'https://example.test/latest');
  assert.equal(stores.get(app.tabs)[0].workspace_id, 'ws-personal');
});

test('initial snapshot returned after disposal cannot replace the current session', async () => {
  let complete;
  const app = domain({ fetchTabs: () => new Promise(resolve => { complete = resolve; }) });
  let active = true;
  const loading = app.initializeTabs(() => active);
  app.applyTabsSnapshot(packet(4, [tab('current', true)]));
  active = false;
  complete(packet(5, [tab('disposed', true)]));
  await loading;
  assert.deepEqual(ids(app.tabs), ['current']);
});

test('late close and navigation replies cannot resurrect tabs after a workspace switch', async () => {
  let complete;
  const app = domain({ closeExistingTab: () => new Promise(done => { complete = done; }) });
  app.applyTabsSnapshot(packet(1, [tab('a', true), tab('b')]));
  const closing = app.removeTab('b');
  app.applyTabsSnapshot(packet(3, [{ ...tab('work', true), workspace_id: 'ws-work' }]));
  complete(packet(2, [tab('a', true)]));
  await closing;
  app.applyNavigationPatch({ id: 'b', url: 'https://example.test/stale', title: 'stale', navigation: { revision: 9 } });
  assert.deepEqual(ids(app.tabs), ['work']);
  assert.equal(stores.get(app.tabs)[0].workspace_id, 'ws-work');
  assert.equal(stores.get(app.currentUrl), 'https://example.test/work');
});

test('reordering can retry after persistence failure without losing newer navigation', async () => {
  let attempts = 0;
  const app = domain({ reorderTabs: async () => {
    if (++attempts === 1) throw new Error('disk full');
    return packet(2, [tab('b'), tab('a', true)]);
  } });
  app.applyTabsSnapshot(packet(1, [tab('a', true), tab('b')]));
  await assert.rejects(app.reorderTabsList(['b', 'a']), /disk full/);
  assert.deepEqual(ids(app.tabs), ['a', 'b']);
  app.applyNavigationPatch({ id: 'a', url: 'https://example.test/latest', title: 'latest', navigation: { revision: 5 } });
  await app.reorderTabsList(['b', 'a']);
  assert.deepEqual(ids(app.tabs), ['b', 'a']);
  assert.equal(stores.get(app.currentUrl), 'https://example.test/latest');
  assert.equal(attempts, 2);
});
