import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

function createWallpaperFixture(initialStorage = {}) {
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
      this.attributes = {};
      this.style = {
        properties: {},
        setProperty(k, v) { this.properties[k] = v; }
      };
      this.classList = {
        classes: new Set(),
        add(c) { this.classes.add(c); },
        remove(c) { this.classes.delete(c); },
        contains(c) { return this.classes.has(c); }
      };
    }
    addEventListener(t, fn) { this.listeners[t] = fn; }
    setAttribute(k, v) { this.attributes[k] = v; }
    getAttribute(k) { return k === 'src' ? this.src : this.attributes[k]; }
    removeAttribute(k) {
      if (k === 'src') delete this.src;
      delete this.attributes[k];
    }
    click() { this.listeners.click?.({ target: this }); }
  }

  const nodes = new Map();
  for (const [, id] of readFileSync('public/new-tab.html', 'utf8').matchAll(/id="([^"]+)"/g)) {
    nodes.set(id, new ElementNode(id));
  }

  const document = {
    readyState: 'complete',
    body: new ElementNode('body'),
    getElementById: id => nodes.get(id) ?? null,
    querySelectorAll: selector => {
      if (selector === '[data-label]') {
        return [...nodes.values()].filter(n => n.dataset && n.dataset.label);
      }
      return [];
    }
  };

  // Mock IndexedDB
  const idbStore = new Map();
  const indexedDB = {
    open: () => {
      const req = {
        result: {
          objectStoreNames: { contains: () => true },
          transaction: () => ({
            objectStore: () => ({
              get: key => {
                const getReq = {};
                setTimeout(() => {
                  getReq.result = idbStore.get(key);
                  getReq.onsuccess?.();
                }, 0);
                return getReq;
              },
              put: (val, key) => {
                const putReq = {};
                idbStore.set(key, val);
                setTimeout(() => putReq.onsuccess?.(), 0);
                return putReq;
              },
              delete: key => {
                const delReq = {};
                idbStore.delete(key);
                setTimeout(() => delReq.onsuccess?.(), 0);
                return delReq;
              }
            })
          })
        }
      };
      setTimeout(() => req.onsuccess?.(), 0);
      return req;
    }
  };

  const script = readFileSync('public/new-tab/wallpaper.js', 'utf8');
  runInNewContext(script, {
    window,
    document,
    localStorage,
    indexedDB,
    console,
    Date,
    parseInt,
    Math,
    CustomEvent: class {
      constructor(type, init) {
        this.type = type;
        this.detail = init?.detail;
      }
    }
  });

  return { nodes, storage, idbStore, window, document, listeners };
}

test('wallpaper starts hidden when no custom image is saved', async () => {
  const f = createWallpaperFixture();
  await new Promise(r => setTimeout(r, 10));

  const container = f.nodes.get('wallpaper-container');
  const toggle = f.nodes.get('wallpaper-enabled');
  assert.equal(container.hidden, true);
  assert.equal(toggle.checked, false);
  assert.equal(f.nodes.get('wallpaper-preview-wrap').hidden, true);
  assert.equal(f.nodes.get('wallpaper-remove').hidden, true);
});

test('wallpaper applies stored image and activates background container with dimming', async () => {
  const testData = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
  const f = createWallpaperFixture({
    veloce_wallpaper_enabled: 'true',
    veloce_wallpaper_dim: '50'
  });
  f.idbStore.set('current', testData);

  // Trigger DOMContentLoaded / init
  for (const fn of f.listeners.get('DOMContentLoaded') || []) fn();
  await new Promise(r => setTimeout(r, 20));

  const container = f.nodes.get('wallpaper-container');
  const image = f.nodes.get('wallpaper-image');
  const overlay = f.nodes.get('wallpaper-overlay');
  const toggle = f.nodes.get('wallpaper-enabled');

  assert.equal(toggle.checked, true);
  assert.equal(container.hidden, false);
  assert.equal(image.src, testData);
  assert.equal(overlay.style.properties['--wallpaper-dim'], '0.5');
  assert.equal(f.document.body.classList.contains('has-wallpaper'), true);
  assert.equal(f.nodes.get('wallpaper-preview-wrap').hidden, false);
  assert.equal(f.nodes.get('wallpaper-preview').src, testData);
  assert.equal(f.nodes.get('wallpaper-remove').hidden, false);
});

test('dimming slider updates dimming value and saves preference', async () => {
  const f = createWallpaperFixture({
    veloce_wallpaper_enabled: 'true',
    veloce_wallpaper_dim: '40'
  });
  await new Promise(r => setTimeout(r, 10));

  const slider = f.nodes.get('wallpaper-dim');
  const dimVal = f.nodes.get('wallpaper-dim-val');
  const overlay = f.nodes.get('wallpaper-overlay');

  slider.listeners.input({ target: { value: '65' } });
  assert.equal(dimVal.textContent, '65%');
  assert.equal(overlay.style.properties['--wallpaper-dim'], '0.65');

  slider.listeners.change({ target: { value: '65' } });
  assert.equal(f.storage.get('veloce_wallpaper_dim'), '65');
});

test('remove button deletes wallpaper and clears DOM presentation', async () => {
  const testData = 'data:image/png;base64,abc';
  const f = createWallpaperFixture({
    veloce_wallpaper_enabled: 'true'
  });
  f.idbStore.set('current', testData);

  await new Promise(r => setTimeout(r, 10));
  const removeBtn = f.nodes.get('wallpaper-remove');
  await removeBtn.listeners.click();
  await new Promise(r => setTimeout(r, 20));

  assert.equal(f.idbStore.has('current'), false);
  assert.equal(f.nodes.get('wallpaper-container').hidden, true);
  assert.equal(f.nodes.get('wallpaper-image').src, undefined);
  assert.equal(f.nodes.get('wallpaper-preview-wrap').hidden, true);
  assert.equal(f.document.body.classList.contains('has-wallpaper'), false);
});

test('storage events from another tab update wallpaper state', async () => {
  const f = createWallpaperFixture({
    veloce_wallpaper_enabled: 'false'
  });
  await new Promise(r => setTimeout(r, 10));

  const container = f.nodes.get('wallpaper-container');
  assert.equal(container.hidden, true);

  // Another tab sets wallpaper and notifies
  f.storage.set('veloce_wallpaper_enabled', 'true');
  f.idbStore.set('current', 'data:image/png;base64,tab2');

  for (const fn of f.listeners.get('storage') || []) {
    fn({ key: 'veloce_wallpaper_updated' });
  }
  await new Promise(r => setTimeout(r, 20));

  assert.equal(container.hidden, false);
  assert.equal(f.nodes.get('wallpaper-image').src, 'data:image/png;base64,tab2');
});
