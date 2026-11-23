import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { bundle, OUTPUT } from '../tools/build-nebula-background.mjs';

test('the script the new tab loads is the current build of the editor engine and never reaches the app', async () => {
  const built = await bundle();
  assert.equal(readFileSync(OUTPUT, 'utf8').replace(/\r\n/g, '\n'), built, 'run `npm run build:nebula`');
  assert.doesNotMatch(built, /tauri|__TAURI|invoke\(/i, 'the new tab page has no bridge to the app');
  assert.match(built, /__sparkNebula/);
});

function element(id = '') {
  const listeners = {};
  const node = {
    id, children: [], hidden: false, style: {}, dataset: {}, attributes: {}, className: '', textContent: '', listeners,
    appendChild(child) { child.parent = node; node.children.push(child); return child; },
    append(...items) { for (const item of items) node.appendChild(item); },
    remove() { if (node.parent) node.parent.children = node.parent.children.filter(child => child !== node); },
    setAttribute(name, value) { node.attributes[name] = String(value); },
    addEventListener(type, listener) { listeners[type] = listener; },
  };
  return node;
}

function fixture(items = []) {
  const store = new Map();
  if (items.length) store.set('spark.wallpaperPresets.v1', JSON.stringify({ version: 1, items }));
  const nodes = Object.fromEntries(['wallpaper-presets', 'custom-background', 'wallpaper-tab-custom'].map(id => [id, element(id)]));
  nodes['wallpaper-presets'].appendChild(element('wallpaper-presets-label'));
  const calls = { start: 0, update: [], destroy: 0, events: [] };
  const window = {
    listeners: {},
    addEventListener(type, listener) { window.listeners[type] = listener; },
    dispatchEvent(event) { calls.events.push(event.type); },
    __sparkNebula: {
      sanitize: value => ({ ...value, checked: true }),
      gradientCss: () => 'linear-gradient(red, blue)',
      start() { calls.start++; return { update: params => calls.update.push(params), destroy() { calls.destroy++; } }; },
    },
  };
  const document = {
    readyState: 'complete',
    getElementById: id => nodes[id] ?? null,
    createElement: () => element(),
    addEventListener() {},
  };
  const localStorage = { getItem: key => store.get(key) ?? null, setItem: (key, value) => store.set(key, String(value)), removeItem: key => store.delete(key) };
  class CustomEvent { constructor(type) { this.type = type; } }
  runInNewContext(readFileSync('public/new-tab/custom-background.js', 'utf8'), { window, document, localStorage, CustomEvent, JSON });
  return { nodes, store, calls, window };
}
const presetsOf = fx => fx.nodes['wallpaper-presets'].children.filter(child => child.id !== 'wallpaper-presets-label');
const saved = [{ id: 'a', name: 'Warm', params: { bias: 0.3 } }, { id: 'b', name: 'Cold', params: { bias: 0.1 } }];

test('with no preset saved the list explains how to make one and nothing is drawn', () => {
  const fx = fixture();
  const [note] = presetsOf(fx);
  assert.equal(note.className, 'settings-note');
  assert.equal(note.textContent, 'wallpaperNoPresets');
  assert.equal(fx.nodes['custom-background'].hidden, true);
  assert.equal(fx.calls.start, 0);
});

test('every saved preset becomes a card with its gradient and name', () => {
  const fx = fixture(saved);
  const cards = presetsOf(fx);
  assert.equal(cards.length, 2);
  assert.equal(cards[0].children[1].textContent, 'Warm');
  assert.equal(cards[0].children[0].style.background, 'linear-gradient(red, blue)');
  assert.deepEqual(cards.map(card => card.attributes['aria-checked']), ['false', 'false']);
  assert.equal(fx.calls.start, 0, 'nothing is drawn until one is chosen');
});

test('choosing a card draws it, choosing it again removes it', () => {
  const fx = fixture(saved);
  presetsOf(fx)[0].listeners.click();
  assert.equal(JSON.parse(fx.store.get('spark_custom_background')).id, 'a');
  assert.equal(fx.calls.start, 1);
  assert.equal(fx.calls.update.at(-1).bias, 0.3);
  assert.equal(fx.nodes['custom-background'].hidden, false);
  assert.deepEqual(fx.calls.events, ['spark-custom-background']);
  assert.deepEqual(presetsOf(fx).map(card => card.attributes['aria-checked']), ['true', 'false']);

  presetsOf(fx)[1].listeners.click();
  assert.equal(JSON.parse(fx.store.get('spark_custom_background')).id, 'b');
  assert.equal(fx.calls.start, 1, 'the same drawing is reused');
  assert.equal(fx.calls.update.at(-1).bias, 0.1);

  presetsOf(fx)[1].listeners.click();
  assert.equal(fx.store.has('spark_custom_background'), false);
  assert.equal(fx.calls.destroy, 1);
  assert.equal(fx.nodes['custom-background'].hidden, true);
  assert.deepEqual(presetsOf(fx).map(card => card.attributes['aria-checked']), ['false', 'false']);
});

test('the chosen background survives a reload and another page can change it', () => {
  const first = fixture(saved);
  presetsOf(first)[0].listeners.click();
  const kept = JSON.parse(first.store.get('spark_custom_background'));
  assert.equal(kept.params.bias, 0.3, 'a copy of the settings is kept with the choice');
  // A new page that finds the choice in storage draws it right away.
  const fx = fixture(saved);
  fx.store.set('spark_custom_background', JSON.stringify(kept));
  fx.window.listeners.storage({ key: 'spark_custom_background' });
  assert.equal(fx.calls.start, 1);
  assert.equal(fx.nodes['custom-background'].hidden, false);
  // A preset saved in the editor shows up without reloading.
  fx.store.set('spark.wallpaperPresets.v1', JSON.stringify({ items: [...saved, { id: 'c', name: 'New', params: {} }] }));
  fx.window.listeners.storage({ key: 'spark.wallpaperPresets.v1' });
  assert.equal(presetsOf(fx).length, 3);
});

test('damaged storage is ignored instead of breaking the page', () => {
  const fx = fixture();
  fx.store.set('spark.wallpaperPresets.v1', '{not json');
  fx.store.set('spark_custom_background', 'also not json');
  fx.window.listeners.storage({ key: 'spark_custom_background' });
  assert.equal(fx.calls.start, 0);
  assert.equal(fx.nodes['custom-background'].hidden, true);
});
