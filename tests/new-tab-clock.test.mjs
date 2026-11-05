import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

function createClockFixture(initialStorage = {}) {
  const storage = new Map(Object.entries(initialStorage));
  const localStorage = {
    getItem: k => storage.get(k) ?? null,
    setItem: (k, v) => storage.set(k, String(v)),
    removeItem: k => storage.delete(k)
  };

  const listeners = new Map();
  const window = {
    addEventListener: (t, fn) => {
      const arr = listeners.get(t) || [];
      arr.push(fn);
      listeners.set(t, arr);
    },
    dispatchEvent: e => {
      for (const fn of listeners.get(e.type) || []) fn(e);
    }
  };

  class ElementNode {
    constructor(id) {
      this.id = id;
      this.children = [];
      this.listeners = {};
      this.value = '';
      this.hidden = false;
      this.checked = false;
      this.textContent = '';
      this.attributes = {};
      this.classList = {
        classes: new Set(),
        add(c) { this.classes.add(c); },
        remove(c) { this.classes.delete(c); },
        contains(c) { return this.classes.has(c); }
      };
    }
    addEventListener(t, fn) { this.listeners[t] = fn; }
    setAttribute(k, v) { this.attributes[k] = v; }
    getAttribute(k) { return this.attributes[k]; }
    click() { this.listeners.click?.({ target: this }); }
  }

  const nodes = new Map();
  const html = readFileSync('public/new-tab.html', 'utf8');
  for (const match of html.matchAll(/<([a-z0-9-]+)[^>]*id="([^"]+)"[^>]*>/gi)) {
    const id = match[2];
    const node = new ElementNode(id);
    if (match[0].includes(' hidden')) node.hidden = true;
    nodes.set(id, node);
  }

  const document = {
    readyState: 'complete',
    body: new ElementNode('body'),
    getElementById: id => nodes.get(id) ?? null,
    querySelectorAll: selector => {
      if (selector === '.clock-widget') {
        return [
          nodes.get('clock-bottom-widget'),
          nodes.get('clock-top-widget'),
          nodes.get('clock-card-widget')
        ].filter(Boolean);
      }
      return [];
    },
    addEventListener: (t, fn) => {
      const arr = listeners.get(t) || [];
      arr.push(fn);
      listeners.set(t, arr);
    }
  };

  const mockDate = new Date(2026, 10, 5, 14, 30, 45); // Nov 5, 2026, 14:30:45 (Thursday)
  class MockDate extends Date {
    constructor(...args) {
      if (args.length) super(...args);
      else super(mockDate.getTime());
    }
  }

  const context = {
    window,
    document,
    localStorage,
    Date: MockDate,
    Intl,
    clearTimeout,
    setTimeout: (fn, ms) => setTimeout(fn, 1),
    CustomEvent: class {
      constructor(t, o) {
        this.type = t;
        this.detail = o?.detail;
      }
    }
  };

  runInNewContext(readFileSync('public/new-tab/clock.js', 'utf8'), context);

  const labels = JSON.parse(readFileSync('src/lib/i18n/catalogs/pt-BR/newtab.json', 'utf8'));
  window.dispatchEvent(new context.CustomEvent('veloce-new-tab', {
    detail: { labels, preferences: { language: 'pt-BR' } }
  }));

  return { nodes, storage, window, listeners, context };
}

test('clock starts enabled by default at bottom position with 24h format', () => {
  const f = createClockFixture();
  const bottomWidget = f.nodes.get('clock-bottom-widget');
  const topWidget = f.nodes.get('clock-top-widget');
  const cardWidget = f.nodes.get('clock-card-widget');
  const time = f.nodes.get('clock-bottom-time');

  assert.equal(bottomWidget.hidden, false);
  assert.equal(topWidget.hidden, true);
  assert.equal(cardWidget.hidden, true);
  assert.equal(time.textContent, '14:30');
  assert.equal(bottomWidget.classList.contains('clock-font-system'), true);
});

test('clock switches to 12h format with AM/PM', () => {
  const f = createClockFixture({ veloce_clock_format: '12h' });
  const time = f.nodes.get('clock-bottom-time');
  assert.equal(time.textContent, '2:30 PM');
});

test('clock includes seconds when seconds toggle is enabled', () => {
  const f = createClockFixture({ veloce_clock_seconds: 'true' });
  const time = f.nodes.get('clock-bottom-time');
  assert.equal(time.textContent, '14:30:45');
});

test('clock changes font family classes', () => {
  const f = createClockFixture({ veloce_clock_font: 'monospace' });
  const bottomWidget = f.nodes.get('clock-bottom-widget');
  assert.equal(bottomWidget.classList.contains('clock-font-monospace'), true);

  const fontSelect = f.nodes.get('clock-font');
  fontSelect.value = 'serif';
  fontSelect.listeners.change?.({ target: fontSelect });

  assert.equal(bottomWidget.classList.contains('clock-font-serif'), true);
  assert.equal(f.storage.get('veloce_clock_font'), 'serif');
});

test('clock changes position to top (above search) and card', () => {
  const f = createClockFixture();
  const posSelect = f.nodes.get('clock-position');

  posSelect.value = 'top';
  posSelect.listeners.change?.({ target: posSelect });

  assert.equal(f.nodes.get('clock-top-widget').hidden, false);
  assert.equal(f.nodes.get('clock-bottom-widget').hidden, true);
  assert.equal(f.nodes.get('clock-top-time').textContent, '14:30');

  posSelect.value = 'card';
  posSelect.listeners.change?.({ target: posSelect });

  assert.equal(f.nodes.get('clock-card-widget').hidden, false);
  assert.equal(f.nodes.get('clock-top-widget').hidden, true);
  assert.equal(f.nodes.get('clock-card-time').textContent, '14:30');
});

test('clock shows greeting when enabled', () => {
  const f = createClockFixture({ veloce_clock_greeting: 'true' });
  const greeting = f.nodes.get('clock-bottom-greeting');
  assert.equal(greeting.hidden, false);
  assert.equal(greeting.textContent, 'Boa tarde');
});

test('clock can be disabled via toggle', () => {
  const f = createClockFixture();
  const toggle = f.nodes.get('clock-enabled');
  assert.equal(toggle.checked, true);

  toggle.checked = false;
  toggle.listeners.change?.({ target: toggle });

  assert.equal(f.nodes.get('clock-bottom-widget').hidden, true);
  assert.equal(f.nodes.get('clock-top-widget').hidden, true);
  assert.equal(f.nodes.get('clock-card-widget').hidden, true);
  assert.equal(f.storage.get('veloce_clock_enabled'), 'false');
});

test('clock synchronizes from storage event in another tab', () => {
  const f = createClockFixture();
  f.storage.set('veloce_clock_format', '12h');

  for (const fn of f.listeners.get('storage') || []) {
    fn({ key: 'veloce_clock_format' });
  }

  assert.equal(f.nodes.get('clock-bottom-time').textContent, '2:30 PM');
});
