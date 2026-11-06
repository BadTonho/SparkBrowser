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
    play() { return Promise.resolve(); }
    pause() {}
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
      if (selector === '[data-label]') {
        return [...nodes.values()].filter(n => n.dataset && n.dataset.label);
      }
      return [];
    }
  };

  // Mock IndexedDB
  const idbStore = new Map();
  const stats = { opens: 0, reads: 0 };
  const indexedDB = {
    open: () => {
      stats.opens++;
      const req = {
        result: {
          objectStoreNames: { contains: () => true },
          transaction: () => ({
            objectStore: () => ({
              get: key => {
                stats.reads++;
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
    URL: {
      createObjectURL: blob => 'blob:http://tauri.localhost/test-blob',
      revokeObjectURL: () => {}
    },
    CustomEvent: class {
      constructor(type, init) {
        this.type = type;
        this.detail = init?.detail;
      }
    }
  });

  return { nodes, storage, idbStore, window, document, listeners, stats };
}

test('wallpaper reads are shared until another tab changes the stored record', async () => {
  const f = createWallpaperFixture({ veloce_wallpaper_enabled: 'true' });
  f.idbStore.set('current', 'data:image/png;base64,one');
  await new Promise(r => setTimeout(r, 20));
  const labels = {};
  f.window.dispatchEvent({ type: 'veloce-new-tab', detail: { labels } });
  f.window.dispatchEvent({ type: 'veloce-new-tab', detail: { labels } });
  await new Promise(r => setTimeout(r, 20));
  assert.equal(f.stats.opens, 1);
  assert.equal(f.stats.reads, 1);
  f.idbStore.set('current', 'data:image/png;base64,two');
  f.window.dispatchEvent({ type: 'storage', key: 'veloce_wallpaper_updated' });
  await new Promise(r => setTimeout(r, 20));
  assert.equal(f.stats.reads, 2);
  assert.equal(f.nodes.get('wallpaper-image').src, 'data:image/png;base64,two');
});

test('images are size-limited and stored as one blob', async () => {
  const f = createWallpaperFixture();
  await new Promise(r => setTimeout(r, 20));
  const labels = JSON.parse(readFileSync('src/lib/i18n/catalogs/en/newtab.json', 'utf8'));
  f.window.dispatchEvent({ type: 'veloce-new-tab', detail: { labels } });
  const input = f.nodes.get('wallpaper-file');
  input.files = [{ name: 'large.png', type: 'image/png', size: 21 * 1024 * 1024 }];
  input.listeners.change();
  assert.match(f.nodes.get('wallpaper-status').textContent, /20 MB/);
  assert.equal(f.idbStore.has('current'), false);
  const image = { name: 'small.png', type: 'image/png', size: 100 };
  input.files = [image];
  input.listeners.change();
  await new Promise(r => setTimeout(r, 20));
  assert.equal(f.idbStore.get('current').blob, image);
  assert.equal('dataUrl' in f.idbStore.get('current'), false);
});

test('wallpaper starts hidden when no custom image is saved', async () => {
  const f = createWallpaperFixture();
  await new Promise(r => setTimeout(r, 25));

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

test('settings menu list opens option detail view and back button returns to list', async () => {
  const f = createWallpaperFixture();
  await new Promise(r => setTimeout(r, 20));

  const mainView = f.nodes.get('settings-main-view');
  const weatherSec = f.nodes.get('weather-section');
  const gmailSec = f.nodes.get('gmail-section');
  const wallpaperSec = f.nodes.get('wallpaper-section');

  const weatherItem = f.nodes.get('menu-item-weather');
  const gmailItem = f.nodes.get('menu-item-gmail');
  const wallpaperItem = f.nodes.get('menu-item-wallpaper');
  const backBtn = f.nodes.get('settings-back');

  // Initial state: main menu visible, detail sections hidden, back hidden
  assert.equal(mainView.hidden, false);
  assert.equal(weatherSec.hidden, true);
  assert.equal(gmailSec.hidden, true);
  assert.equal(wallpaperSec.hidden, true);
  assert.equal(backBtn.hidden, true);
  assert.ok(weatherItem);
  assert.ok(gmailItem);
  assert.ok(wallpaperItem);
});

test('wallpaper applies stored video and plays in wallpaper-video element', async () => {
  const f = createWallpaperFixture({
    veloce_wallpaper_enabled: 'true',
    veloce_wallpaper_dim: '30'
  });
  f.idbStore.set('current', {
    mediaType: 'video',
    mime: 'video/mp4',
    blob: { type: 'video/mp4', size: 1024 }
  });

  // Trigger DOMContentLoaded
  for (const fn of f.listeners.get('DOMContentLoaded') || []) fn();
  await new Promise(r => setTimeout(r, 25));

  const container = f.nodes.get('wallpaper-container');
  const video = f.nodes.get('wallpaper-video');
  const image = f.nodes.get('wallpaper-image');
  const previewWrap = f.nodes.get('wallpaper-preview-wrap');
  const videoPreview = f.nodes.get('wallpaper-video-preview');

  assert.equal(container.hidden, false);
  assert.equal(video.hidden, false);
  assert.equal(video.src, 'blob:http://tauri.localhost/test-blob');
  assert.equal(image.hidden, true);
  assert.equal(image.src, undefined);
  assert.equal(previewWrap.hidden, false);
  assert.equal(videoPreview.hidden, false);
  assert.equal(videoPreview.src, 'blob:http://tauri.localhost/test-blob');
});

test('video file size limit rejects files larger than 50MB', async () => {
  const f = createWallpaperFixture({
    veloce_wallpaper_enabled: 'true'
  });
  await new Promise(r => setTimeout(r, 20));

  const labels = JSON.parse(readFileSync('src/lib/i18n/catalogs/pt-BR/newtab.json', 'utf8'));
  f.window.dispatchEvent({ type: 'veloce-new-tab', detail: { labels, preferences: { language: 'pt-BR' } } });

  const fileInput = f.nodes.get('wallpaper-file');
  const status = f.nodes.get('wallpaper-status');

  // Simulate file > 50MB
  fileInput.files = [{
    name: 'huge-video.mp4',
    type: 'video/mp4',
    size: 55 * 1024 * 1024
  }];

  fileInput.listeners.change();
  await new Promise(r => setTimeout(r, 20));

  assert.match(status.textContent, /50 MB/i);
  assert.equal(f.idbStore.has('current'), false);
});
