import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { bundle, OUTPUT } from '../tools/build-new-tab-suggestions.mjs';

const read = path => readFileSync(path, 'utf8').replace(/\r\n/g, '\n');

test('the ranking script of the new tab page matches the address bar source', async () => {
  assert.equal(read(OUTPUT), await bundle(), 'run `npm run build:suggestions`');
});

function fixture({ storage = {} } = {}) {
  class Node {
    constructor() { this.children = []; this.attributes = {}; this.listeners = {}; this.value = ''; this.hidden = true; }
    addEventListener(type, fn) { this.listeners[type] = fn; }
    setAttribute(name, value) { this.attributes[name] = value; }
    removeAttribute(name) { delete this.attributes[name]; }
    replaceChildren(...nodes) { this.children = nodes; }
    append(...nodes) { this.children.push(...nodes); }
    fire(type, event = {}) { let prevented = false; this.listeners[type]?.({ preventDefault() { prevented = true; }, ...event }); return prevented; }
  }
  const nodes = { '#search': new Node(), '#suggestions': new Node(), '#search-form': new Node() };
  const document = { querySelector: id => nodes[id], createElement: () => new Node(), createElementNS: () => new Node() };
  const listeners = {}, sent = [], timers = new Map();
  let timer = 0;
  const window = { addEventListener: (type, fn) => { listeners[type] = fn; }, dispatchEvent: event => sent.push({ ...event.detail }) };
  const context = {
    window, document, URL,
    localStorage: { getItem: key => storage[key] ?? null },
    setTimeout: fn => { timers.set(++timer, fn); return timer; },
    clearTimeout: id => timers.delete(id),
    CustomEvent: class { constructor(type, init) { this.type = type; this.detail = init.detail; } },
  };
  runInNewContext(read('public/new-tab/suggestions-core.js') + ';window.SparkSuggestions=SparkSuggestions;\n' + read('public/new-tab/suggestions.js'), context);
  const labels = JSON.parse(read('src/lib/i18n/catalogs/en/newtab.json'));
  const present = (patch = {}) => listeners['spark-new-tab']({ detail: { labels, preferences: { engine: 'google' }, ...patch } });
  present();
  return {
    input: nodes['#search'], list: nodes['#suggestions'], sent, present,
    run() { const pending = [...timers.values()]; timers.clear(); pending.forEach(fn => fn()); },
    reply: detail => listeners['spark-new-tab-suggestions']({ detail }),
  };
}

const local = {
  history: [{ id: 1, title: 'Rust book', url: 'https://doc.test/book', visited_at: 1 }],
  bookmarks: [], icons: {}, sites: {},
};
const answer = f => f.reply({ request: f.sent.at(-1).request, local });
const primaries = f => f.list.children.map(row => row.children[1].children[0].textContent);

test('a click on the field lists recent sites; automatic focus does not', () => {
  const f = fixture();
  assert.equal(f.input.listeners.focus, undefined);
  f.input.fire('pointerdown');
  assert.equal(f.sent.at(-1).action, 'suggest');
  assert.equal(f.sent.at(-1).input, '');
  answer(f);
  assert.equal(f.list.hidden, false);
  assert.deepEqual(primaries(f), ['Rust book']);
  f.input.fire('blur');
  assert.equal(f.list.hidden, true);
});

test('typing asks after a pause, then for engine terms, and drops late replies', () => {
  const f = fixture();
  f.input.value = 'ru'; f.input.fire('input');
  f.input.value = 'rust'; f.input.fire('input');
  assert.equal(f.sent.length, 0);
  f.run();
  assert.deepEqual(f.sent.map(item => item.input), ['rust']);
  const stale = f.sent.at(-1).request - 1;
  f.reply({ request: stale, local });
  assert.equal(f.list.hidden, true);
  answer(f); f.run();
  assert.equal(f.sent.at(-1).action, 'terms');
  f.reply({ request: f.sent.at(-1).request, terms: ['rust lang'] });
  assert.deepEqual(primaries(f), ['rust', 'Rust book', 'rust lang']);
});

test('no terms are asked for in a private window, with the option off or for an address', () => {
  const cases = [
    [fixture(), 'rust', { private: true }],
    [fixture({ storage: { spark_web_suggestions: 'false' } }), 'rust', {}],
    [fixture(), 'example.com', {}],
    [fixture(), 'rust', { preferences: { engine: 'kagi' } }],
  ];
  for (const [f, text, patch] of cases) {
    f.present(patch);
    f.input.value = text; f.input.fire('input'); f.run(); answer(f); f.run();
    assert.deepEqual(f.sent.map(item => item.action), ['suggest'], text);
  }
});

test('arrows highlight a row, Enter opens it and Escape closes the list', () => {
  const f = fixture();
  f.input.value = 'rust'; f.input.fire('input'); f.run(); answer(f);
  assert.equal(f.input.fire('keydown', { key: 'Enter' }), false, 'with nothing highlighted the form submits');
  f.input.fire('keydown', { key: 'ArrowDown' });
  f.input.fire('keydown', { key: 'ArrowDown' });
  assert.equal(f.list.children[1].className, 'suggestion highlighted');
  assert.equal(f.input.fire('keydown', { key: 'Enter' }), true);
  assert.equal(f.sent.at(-1).action, 'navigate');
  assert.equal(f.sent.at(-1).input, 'https://doc.test/book');
  assert.equal(f.list.hidden, true);
  f.input.fire('keydown', { key: 'ArrowDown' }); answer(f);
  f.input.fire('keydown', { key: 'Escape' });
  assert.equal(f.list.hidden, true);
});
