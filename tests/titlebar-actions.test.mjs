import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

function load(path, imports = {}, globals = {}) {
  const source = ts.transpileModule(readFileSync(path, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, { exports, require: id => {
    if (!(id in imports)) throw Error('Missing import: ' + id);
    return imports[id];
  }, ...globals });
  return exports;
}
const plain = value => JSON.parse(JSON.stringify(value));
const defaults = ['tools', 'bookmarks', 'commands', 'history', 'downloads', 'privacy', 'settings', 'zen'];

function storeFixture(saved = null, failWrite = false) {
  const storage = new Map(saved == null ? [] : [['veloce_titlebar_action_order', saved]]);
  const writable = value => ({ set(next) { value = next; }, subscribe(fn) { fn(value); return () => {}; } });
  const get = store => { let value; store.subscribe(next => value = next)(); return value; };
  const store = load('src/lib/stores/titlebarActionsStore.ts', {
    'svelte/store': { get, writable }, '../services/storage.js': { safeStorage: {
      getItem: key => storage.get(key) ?? null,
      setItem: (key, value) => { if (failWrite) return false; storage.set(key, value); return true; },
    } },
  });
  return { ...store, storage, current: () => plain(get(store.titlebarActionOrder)) };
}

test('order defaults, validates unknown input, deduplicates and restores missing actions', () => {
  for (const input of [null, '{', '{}', 'null', '42']) assert.deepEqual(storeFixture(input).current(), defaults);
  assert.deepEqual(storeFixture('["privacy","invalid","privacy",9]').current(), ['privacy', ...defaults.filter(id => id !== 'privacy')]);
});

test('visible actions move to either end, persist and reload', () => {
  const f = storeFixture();
  f.moveTitlebarAction('zen', 'tools', defaults);
  assert.equal(f.current()[0], 'zen');
  f.moveTitlebarAction('tools', null, defaults);
  assert.equal(f.current().at(-1), 'tools');
  assert.deepEqual(storeFixture(f.storage.get('veloce_titlebar_action_order')).current(), f.current());
});

test('hidden bookmarks retain their saved slot; unavailable storage still updates the session', () => {
  const f = storeFixture(null, true);
  f.moveTitlebarAction('zen', 'tools', defaults.filter(id => id !== 'bookmarks'));
  assert.equal(f.current()[0], 'zen');
  assert.equal(f.current()[1], 'bookmarks');
  assert.equal(f.storage.size, 0);
  const before = f.current();
  f.moveTitlebarAction('bookmarks', null, defaults.filter(id => id !== 'bookmarks'));
  assert.deepEqual(f.current(), before);
});

function dragFixture() {
  let now = 0, callback, due, preview;
  const commits = [], captures = [], releases = [];
  const { createActionDrag } = load('src/lib/services/titlebar-action-drag.ts', {}, {
    setTimeout: (fn, ms) => { callback = fn; due = now + ms; return 1; }, clearTimeout: () => { callback = null; },
  });
  const point = (x = 15, y = 16, pointerId = 1) => ({ pointerId, clientX: x, clientY: y });
  const drag = createActionDrag({
    items: () => defaults.map((id, i) => ({ id, left: i * 30, right: i * 30 + 27, top: 0, bottom: 32 })),
    bounds: () => ({ left: 0, right: 240, top: 0, bottom: 32 }),
    capture: id => captures.push(id), release: id => releases.push(id),
    preview: value => preview = value, commit: (...args) => commits.push(plain(args)),
  });
  return { drag, point, commits, captures, releases, preview: () => preview,
    down: (id = 'tools', x = 15) => drag.down(id, { ...point(x), button: 0, isPrimary: true }),
    tick(ms) { now += ms; if (callback && now >= due) { const fn = callback; callback = null; fn(); } },
  };
}

test('quick click stays available; hold activates at 500ms and consumes the release click', () => {
  const f = dragFixture();
  f.down(); f.tick(499);
  assert.equal(f.preview(), undefined);
  f.drag.up(f.point());
  assert.equal(f.drag.consumeClick({ detail: 1 }), false);
  f.down(); f.tick(500);
  assert.equal(f.preview().id, 'tools');
  f.drag.up(f.point());
  assert.equal(f.commits.length, 0);
  assert.equal(f.drag.consumeClick({ detail: 1 }), true);
  assert.equal(f.drag.consumeClick({ detail: 0 }), false, 'keyboard activation remains available');
  f.down(); f.drag.up(f.point());
  assert.equal(f.drag.consumeClick({ detail: 1 }), false, 'next quick click works');
});

test('a held action can move to either end, with a visible insertion marker', () => {
  for (const [id, start, end, before] of [['tools', 15, 239, null], ['zen', 225, 1, 'tools']]) {
    const f = dragFixture(); f.down(id, start); f.tick(500); f.drag.move(f.point(end));
    assert.equal(f.preview().before, before);
    assert.equal(typeof f.preview().marker, 'number');
    assert.equal(f.commits.length, 0, 'preview does not persist');
    f.drag.up(f.point(end));
    assert.deepEqual(f.commits, [[id, before, defaults]]);
    assert.deepEqual(f.captures, [1]); assert.deepEqual(f.releases, [1]);
  }
});

test('more than 6px before activation cancels the hold and blocks the accidental click', () => {
  const f = dragFixture(); f.down(); f.drag.move(f.point(21)); f.tick(499);
  assert.equal(f.preview(), undefined);
  f.drag.move(f.point(22)); f.tick(1000); f.drag.up(f.point(239));
  assert.equal(f.preview(), null); assert.equal(f.commits.length, 0);
  assert.equal(f.drag.consumeClick({ detail: 1 }), true);
});

test('outside drop, Escape/blur/capture cancellation and pending cancellation never commit', () => {
  for (const mode of ['outside', 'cancel', 'pending']) {
    const f = dragFixture(); f.down();
    if (mode !== 'pending') f.tick(500);
    if (mode === 'outside') { f.drag.move(f.point(239, 50)); assert.equal(f.preview().marker, null); f.drag.up(f.point(239, 50)); }
    else f.drag.cancel();
    f.tick(1000); f.drag.up(f.point(239));
    assert.equal(f.drag.isTracking(), false); assert.equal(f.preview(), null);
    assert.equal(f.commits.length, 0); assert.equal(f.drag.consumeClick({ detail: 1 }), true);
  }
});

test('secondary buttons and unrelated pointers cannot start or complete the gesture', () => {
  const f = dragFixture();
  f.drag.down('tools', { ...f.point(), button: 2, isPrimary: true }); f.tick(500);
  assert.equal(f.drag.isTracking(), false);
  f.down(); f.tick(500); f.drag.up(f.point(239, 16, 2));
  assert.equal(f.drag.isTracking(), true); assert.equal(f.commits.length, 0);
  f.drag.cancel();
});
