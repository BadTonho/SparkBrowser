import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { setImmediate as flushWallpaper, setTimeout as delay } from 'node:timers/promises';
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
        remove(...names) { for (const name of names) this.classes.delete(name); },
        toggle(c, enabled) { if (enabled) this.add(c); else this.remove(c); },
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
    focus() { this.focused = true; }
    setPointerCapture(id) { this.pointer = id; }
    hasPointerCapture(id) { return this.pointer === id; }
    releasePointerCapture() { this.pointer = undefined; }
    getBoundingClientRect() { return { width: this.clientWidth ?? 0, height: this.clientHeight ?? 0 }; }
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

  // Complete mocked IndexedDB requests asynchronously without depending on timer resolution.
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
                queueMicrotask(() => {
                  getReq.result = idbStore.get(key);
                  getReq.onsuccess?.();
                });
                return getReq;
              },
              put: (val, key) => {
                const putReq = {};
                idbStore.set(key, val);
                queueMicrotask(() => putReq.onsuccess?.());
                return putReq;
              },
              delete: key => {
                const delReq = {};
                idbStore.delete(key);
                queueMicrotask(() => delReq.onsuccess?.());
                return delReq;
              }
            })
          })
        }
      };
      queueMicrotask(() => req.onsuccess?.());
      return req;
    }
  };

  const scripts = ['wallpaper-framing.js', 'wallpaper.js'];
  runInNewContext(scripts.map(file => readFileSync(`public/new-tab/${file}`, 'utf8')).join('\n'), {
    window,
    document,
    localStorage,
    indexedDB,
    console,
    Date,
    setTimeout,
    clearTimeout,
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
  const f = createWallpaperFixture({ spark_wallpaper_enabled: 'true' });
  f.idbStore.set('current', 'data:image/png;base64,one');
  await flushWallpaper();
  const labels = {};
  f.window.dispatchEvent({ type: 'spark-new-tab', detail: { labels } });
  f.window.dispatchEvent({ type: 'spark-new-tab', detail: { labels } });
  await flushWallpaper();
  assert.equal(f.stats.opens, 1);
  assert.equal(f.stats.reads, 1);
  f.idbStore.set('current', 'data:image/png;base64,two');
  f.window.dispatchEvent({ type: 'storage', key: 'spark_wallpaper_updated' });
  await flushWallpaper();
  assert.equal(f.stats.reads, 2);
  assert.equal(f.nodes.get('wallpaper-image').src, 'data:image/png;base64,two');
});

test('images are size-limited and replacing media resets framing', async () => {
  const f = createWallpaperFixture({ spark_wallpaper_zoom: '200', spark_wallpaper_pos: '33,80' });
  await flushWallpaper();
  const labels = JSON.parse(readFileSync('src/lib/i18n/catalogs/en/newtab.json', 'utf8'));
  f.window.dispatchEvent({ type: 'spark-new-tab', detail: { labels } });
  const input = f.nodes.get('wallpaper-file');
  input.files = [{ name: 'large.png', type: 'image/png', size: 21 * 1024 * 1024 }];
  input.listeners.change();
  assert.match(f.nodes.get('wallpaper-status').textContent, /20 MB/);
  assert.equal(f.idbStore.has('current'), false);
  const image = { name: 'small.png', type: 'image/png', size: 100 };
  input.files = [image];
  input.listeners.change();
  await flushWallpaper();
  assert.equal(f.idbStore.get('current').blob, image);
  assert.equal('dataUrl' in f.idbStore.get('current'), false);
  assert.equal(f.storage.get('spark_wallpaper_zoom'), '100');
  assert.equal(f.storage.get('spark_wallpaper_pos'), '50,50');
});

test('wallpaper starts hidden when no custom image is saved', async () => {
  const f = createWallpaperFixture();
  await flushWallpaper();

  const container = f.nodes.get('wallpaper-container');
  const toggle = f.nodes.get('wallpaper-enabled');
  assert.equal(container.hidden, true);
  assert.equal(toggle.checked, false);
  assert.equal(f.nodes.get('wallpaper-editor').hidden, true);
  assert.equal(f.nodes.get('wallpaper-remove').hidden, true);
});

test('wallpaper applies stored image and activates background container with dimming', async () => {
  const testData = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
  const f = createWallpaperFixture({
    spark_wallpaper_enabled: 'true',
    spark_wallpaper_dim: '50'
  });
  f.idbStore.set('current', testData);

  // Trigger DOMContentLoaded / init
  for (const fn of f.listeners.get('DOMContentLoaded') || []) fn();
  await flushWallpaper();

  const container = f.nodes.get('wallpaper-container');
  const image = f.nodes.get('wallpaper-image');
  const overlay = f.nodes.get('wallpaper-overlay');
  const toggle = f.nodes.get('wallpaper-enabled');

  assert.equal(toggle.checked, true);
  assert.equal(container.hidden, false);
  assert.equal(image.src, testData);
  assert.equal(overlay.style.properties['--wallpaper-dim'], '0.5');
  assert.equal(f.document.body.classList.contains('has-wallpaper'), true);
  assert.equal(f.nodes.get('wallpaper-editor').hidden, false);
  assert.equal(f.nodes.get('wallpaper-preview').src, testData);
  assert.equal(f.nodes.get('wallpaper-remove').hidden, false);
});

test('dimming slider updates dimming value and saves preference', async () => {
  const f = createWallpaperFixture({
    spark_wallpaper_enabled: 'true',
    spark_wallpaper_dim: '40'
  });
  await flushWallpaper();

  const slider = f.nodes.get('wallpaper-dim');
  const dimVal = f.nodes.get('wallpaper-dim-val');
  const overlay = f.nodes.get('wallpaper-overlay');
  const previewOverlay = f.nodes.get('wallpaper-preview-overlay');

  slider.listeners.input({ target: { value: '65' } });
  assert.equal(dimVal.textContent, '65%');
  assert.equal(overlay.style.properties['--wallpaper-dim'], '0.65');
  assert.equal(previewOverlay.style.properties['--wallpaper-dim'], '0.65');

  slider.listeners.change({ target: { value: '65' } });
  assert.equal(f.storage.get('spark_wallpaper_dim'), '65');

  f.storage.set('spark_wallpaper_dim', '0');
  f.window.dispatchEvent({ type: 'storage', key: 'spark_wallpaper_dim' });
  await flushWallpaper();
  assert.equal(dimVal.textContent, '0%');
  assert.equal(overlay.style.properties['--wallpaper-dim'], '0');
  assert.equal(previewOverlay.style.properties['--wallpaper-dim'], '0');
});

test('remove button deletes wallpaper and clears DOM presentation', async () => {
  const testData = 'data:image/png;base64,abc';
  const f = createWallpaperFixture({
    spark_wallpaper_enabled: 'true'
  });
  f.idbStore.set('current', testData);

  await flushWallpaper();
  const removeBtn = f.nodes.get('wallpaper-remove');
  await removeBtn.listeners.click();
  await flushWallpaper();

  assert.equal(f.idbStore.has('current'), false);
  assert.equal(f.nodes.get('wallpaper-container').hidden, true);
  assert.equal(f.nodes.get('wallpaper-image').src, undefined);
  assert.equal(f.nodes.get('wallpaper-editor').hidden, true);
  assert.equal(f.document.body.classList.contains('has-wallpaper'), false);
});

test('storage events from another tab update wallpaper state', async () => {
  const f = createWallpaperFixture({
    spark_wallpaper_enabled: 'false'
  });
  await flushWallpaper();

  const container = f.nodes.get('wallpaper-container');
  assert.equal(container.hidden, true);

  // Another tab sets wallpaper and notifies
  f.storage.set('spark_wallpaper_enabled', 'true');
  f.idbStore.set('current', 'data:image/png;base64,tab2');

  for (const fn of f.listeners.get('storage') || []) {
    fn({ key: 'spark_wallpaper_updated' });
  }
  await flushWallpaper();

  assert.equal(container.hidden, false);
  assert.equal(f.nodes.get('wallpaper-image').src, 'data:image/png;base64,tab2');
});

test('settings menu list opens option detail view and back button returns to list', async () => {
  const f = createWallpaperFixture();
  await flushWallpaper();

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
    spark_wallpaper_enabled: 'true',
    spark_wallpaper_dim: '30'
  });
  f.idbStore.set('current', {
    mediaType: 'video',
    mime: 'video/mp4',
    blob: { type: 'video/mp4', size: 1024 }
  });

  // Trigger DOMContentLoaded
  for (const fn of f.listeners.get('DOMContentLoaded') || []) fn();
  await flushWallpaper();

  const container = f.nodes.get('wallpaper-container');
  const video = f.nodes.get('wallpaper-video');
  const image = f.nodes.get('wallpaper-image');
  const editor = f.nodes.get('wallpaper-editor');
  const videoPreview = f.nodes.get('wallpaper-video-preview');

  assert.equal(container.hidden, false);
  assert.equal(video.hidden, false);
  assert.equal(video.src, 'blob:http://tauri.localhost/test-blob');
  assert.equal(image.hidden, true);
  assert.equal(image.src, undefined);
  assert.equal(editor.hidden, false);
  assert.equal(videoPreview.hidden, false);
  assert.equal(videoPreview.src, 'blob:http://tauri.localhost/test-blob');

  Object.assign(container, { clientWidth: 800, clientHeight: 600 });
  Object.assign(f.nodes.get('wallpaper-preview-wrap'), { clientWidth: 400, clientHeight: 300 });
  for (const node of [video, videoPreview]) Object.assign(node, { videoWidth: 1600, videoHeight: 900 });
  videoPreview.listeners.loadedmetadata();
  assert.equal(Number.parseFloat(video.style.left), 2 * Number.parseFloat(videoPreview.style.left));
  assert.equal(Number.parseFloat(video.style.width), 2 * Number.parseFloat(videoPreview.style.width));
});

test('video file size limit rejects files larger than 50MB', async () => {
  const f = createWallpaperFixture({
    spark_wallpaper_enabled: 'true'
  });
  await flushWallpaper();

  const labels = JSON.parse(readFileSync('src/lib/i18n/catalogs/pt-BR/newtab.json', 'utf8'));
  f.window.dispatchEvent({ type: 'spark-new-tab', detail: { labels, preferences: { language: 'pt-BR' } } });

  const fileInput = f.nodes.get('wallpaper-file');
  const status = f.nodes.get('wallpaper-status');

  // Simulate file > 50MB
  fileInput.files = [{
    name: 'huge-video.mp4',
    type: 'video/mp4',
    size: 55 * 1024 * 1024
  }];

  fileInput.listeners.change();
  await flushWallpaper();

  assert.match(status.textContent, /50 MB/i);
  assert.equal(f.idbStore.has('current'), false);
});

test('panel and widget layout choices are segmented radios, validated, persisted locally and default to corner and sides',async()=>{
  const {readFileSync}=await import('node:fs');const {runInNewContext}=await import('node:vm');
  const run=stored=>{
    const store=new Map(Object.entries(stored??{})),listeners=new Map();
    const radio=value=>({value,checked:false,addEventListener(n,f){this.fn=f;}});
    const element=()=>({dataset:{}});
    const radios={panel:[radio('corner'),radio('center')],widgets:[radio('sides'),radio('bottom')]};
    const nodes={'settings-panel':element(),widgets:element(),'settings-layout':{querySelectorAll:()=>radios.panel},'widgets-layout':{querySelectorAll:()=>radios.widgets}};
    runInNewContext(readFileSync('public/new-tab/settings-layout.js','utf8'),{document:{getElementById:id=>nodes[id]},window:{addEventListener:(n,f)=>listeners.set(n,[...(listeners.get(n)??[]),f])},localStorage:{getItem:k=>store.get(k)??null,setItem:(k,v)=>store.set(k,v)}});
    const checked=group=>radios[group].filter(r=>r.checked).map(r=>r.value);
    const pick=(group,value)=>{for(const r of radios[group])r.checked=r.value===value;radios[group].find(r=>r.value===value).fn();};
    return {nodes,store,listeners,radios,checked,pick};
  };
  let f=run();
  assert.equal(f.nodes['settings-panel'].dataset.layout,'corner');assert.equal(f.nodes.widgets.dataset.layout,'sides');
  assert.deepEqual(f.checked('panel'),['corner']);assert.deepEqual(f.checked('widgets'),['sides']);
  f=run({spark_settings_panel_layout:'center',spark_widgets_layout:'bottom'});
  assert.equal(f.nodes['settings-panel'].dataset.layout,'center');assert.equal(f.nodes.widgets.dataset.layout,'bottom');
  assert.deepEqual(f.checked('panel'),['center']);assert.deepEqual(f.checked('widgets'),['bottom']);
  f=run({spark_settings_panel_layout:'<script>',spark_widgets_layout:'top'});
  assert.equal(f.nodes['settings-panel'].dataset.layout,'corner');assert.equal(f.nodes.widgets.dataset.layout,'sides');
  f=run();f.pick('widgets','bottom');
  assert.equal(f.nodes.widgets.dataset.layout,'bottom');assert.equal(f.store.get('spark_widgets_layout'),'bottom');
  assert.deepEqual(f.checked('widgets'),['bottom'],'exactly one option is selected');
  assert.equal(f.store.get('spark_settings_panel_layout'),undefined,'only the changed choice is stored');
  f.radios.widgets[1].value='nonsense';f.radios.widgets[1].fn();
  assert.equal(f.nodes.widgets.dataset.layout,'sides','an unknown value falls back');
  f.radios.widgets[1].value='bottom';
  f.store.set('spark_widgets_layout','bottom');for(const fn of f.listeners.get('storage'))fn({key:'spark_widgets_layout'});
  assert.equal(f.nodes.widgets.dataset.layout,'bottom','another new tab changed it');
  assert.deepEqual(f.checked('widgets'),['bottom']);
  const page=readFileSync('public/new-tab.html','utf8');
  for(const [name,values] of [['panel-layout',['corner','center']],['widgets-layout',['sides','bottom']]]){
    const found=[...page.matchAll(new RegExp('<input type="radio" name="'+name+'" value="([a-z]+)"','g'))].map(m=>m[1]);
    assert.deepEqual(found,values,name+' shows every option inline');
  }
  assert.ok(!/<select id="(settings|widgets)-layout"/.test(page),'no drop-downs for these two choices');
  const css=readFileSync('public/new-tab/settings-layout.css','utf8');
  assert.match(css,/#widgets\[data-layout="bottom"\] #media-widgets\s*\{[^}]*position:fixed[^}]*bottom:20px/);
  assert.match(css,/#media-widgets\s*\{\s*display:contents/,'transparent in the default layout');
  assert.doesNotMatch(css,/#widgets\[data-layout="bottom"\]\s*\{/,'the whole widget area is not moved');
  const html=readFileSync('public/new-tab.html','utf8');
  const start=html.indexOf('id="media-widgets"');
  const group=html.slice(start,html.indexOf('</section>',html.indexOf('id="youtube-widget"')));
  assert.ok(start>0&&group.includes('id="spotify-widget"')&&group.includes('id="youtube-widget"'),'both music cards are grouped');
  assert.ok(!group.includes('weather-widget')&&!group.includes('gmail-widget')&&!group.includes('clock-card-widget'),'other widgets stay put');
});

test('saved alignment applies to both spare space and overflow at different zoom levels', async () => {
  const testData = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
  const f = createWallpaperFixture({ spark_wallpaper_enabled: 'true', spark_wallpaper_zoom: '50', spark_wallpaper_pos: '100,0' });
  f.idbStore.set('current', testData);
  const image = f.nodes.get('wallpaper-image'), container = f.nodes.get('wallpaper-container');
  Object.assign(container, { clientWidth: 1000, clientHeight: 500 });
  Object.assign(image, { naturalWidth: 1000, naturalHeight: 1000 });
  await flushWallpaper();
  // At 100% a square picture covers a 1000x500 area, so it is 1000x1000 and cropped vertically.
  // At 50% it is 500x500: fully visible and aligned to the right by the saved x = 100%.
  assert.equal(image.style.width, '500px');
  assert.equal(image.style.height, '500px');
  assert.equal(image.style.left, '500px');
  assert.equal(image.style.top, '0px');
  assert.equal(image.style.objectFit, 'fill');
  f.storage.set('spark_wallpaper_zoom', '100');
  f.window.dispatchEvent({ type: 'storage', key: 'spark_wallpaper_zoom' });
  await flushWallpaper();
  assert.equal(image.style.width, '1000px');
  assert.equal(image.style.height, '1000px');
  assert.equal(image.style.left, '0px');
  assert.equal(image.style.top, '0px');
});

test('a smaller image reaches all four corners and restores its alignment in another tab', async () => {
  const preferences = { spark_wallpaper_enabled: 'true', spark_wallpaper_zoom: '70' };
  function setup(fixture) {
    fixture.idbStore.set('current', 'data:image/png;base64,landscape');
    const frame = fixture.nodes.get('wallpaper-preview-wrap');
    const preview = fixture.nodes.get('wallpaper-preview');
    const image = fixture.nodes.get('wallpaper-image');
    Object.assign(frame, { clientWidth: 400, clientHeight: 200 });
    Object.assign(fixture.nodes.get('wallpaper-container'), { clientWidth: 1000, clientHeight: 500 });
    for (const node of [preview, image]) Object.assign(node, { naturalWidth: 1000, naturalHeight: 500 });
    return { frame, preview, image };
  }

  const f = createWallpaperFixture(preferences);
  const { frame, preview, image } = setup(f);
  await flushWallpaper();
  assert.equal(preview.style.left, '60px');
  assert.equal(preview.style.top, '30px');

  for (const [x, y] of [[0, 0], [100, 0], [0, 100], [100, 100]]) {
    frame.listeners.pointerdown({ button: 0, pointerId: 1, clientX: 200, clientY: 100, preventDefault() {} });
    frame.listeners.pointermove({ pointerId: 1, clientX: x ? 450 : -50, clientY: y ? 250 : -50, shiftKey: true });
    assert.equal(preview.style.left, x ? '120px' : '0px');
    assert.equal(preview.style.top, y ? '60px' : '0px');
    assert.equal(image.style.left, x ? '300px' : '0px');
    assert.equal(image.style.top, y ? '150px' : '0px');
    frame.listeners.pointerup({ pointerId: 1, shiftKey: true });
    assert.equal(f.storage.get('spark_wallpaper_pos'), `${x},${y}`);
  }

  const other = createWallpaperFixture(Object.fromEntries(f.storage));
  const reopened = setup(other);
  await flushWallpaper();
  assert.equal(reopened.image.style.left, '300px');
  assert.equal(reopened.image.style.top, '150px');
  other.storage.set('spark_wallpaper_pos', '0,0');
  other.window.dispatchEvent({ type: 'storage', key: 'spark_wallpaper_pos' });
  assert.equal(reopened.image.style.left, '0px');
  assert.equal(reopened.image.style.top, '0px');
});

test('wheel zoom and pointer drag keep preview and background in sync and save the framing', async () => {
  const f = createWallpaperFixture({ spark_wallpaper_enabled: 'true', spark_wallpaper_pos: '80,80' });
  f.idbStore.set('current', 'data:image/png;base64,portrait');
  const frame = f.nodes.get('wallpaper-preview-wrap');
  const background = f.nodes.get('wallpaper-container');
  const image = f.nodes.get('wallpaper-image');
  const preview = f.nodes.get('wallpaper-preview');
  Object.assign(background, { clientWidth: 1000, clientHeight: 500 });
  Object.assign(frame, { clientWidth: 400, clientHeight: 200 });
  for (const node of [image, preview]) Object.assign(node, { naturalWidth: 1000, naturalHeight: 1250 });
  await flushWallpaper();

  let prevented = 0;
  const preventDefault = () => prevented++;
  frame.listeners.wheel({ deltaY: -20, deltaMode: 0, preventDefault });
  frame.listeners.wheel({ deltaY: -20, deltaMode: 0, preventDefault });
  assert.equal(prevented, 2);
  assert.equal(f.nodes.get('wallpaper-zoom-val').textContent, '105%');
  assert.equal(image.style.top, '-650px');
  assert.equal(preview.style.top, '-260px');
  assert.equal(f.storage.has('spark_wallpaper_zoom'), false);
  await delay(200);
  assert.equal(f.storage.get('spark_wallpaper_zoom'), '105');
  assert.equal(f.storage.get('spark_wallpaper_pos'), '80,80');

  frame.listeners.pointerdown({ button: 0, pointerId: 1, clientX: 120, clientY: 100, preventDefault });
  frame.listeners.pointermove({ pointerId: 1, clientX: 300, clientY: 165 });
  assert.equal(frame.focused, true);
  assert.equal(preview.style.left, '0px');
  assert.equal(image.style.top, '-487.5px');
  assert.equal(preview.style.top, '-195px');
  assert.equal(f.storage.get('spark_wallpaper_pos'), '80,80');
  frame.listeners.pointerup({ pointerId: 1 });
  assert.equal(f.storage.get('spark_wallpaper_pos'), '0,60');

  Object.assign(background, { clientWidth: 500, clientHeight: 1000 });
  Object.assign(frame, { clientWidth: 150, clientHeight: 300 });
  f.window.dispatchEvent({ type: 'resize' });
  assert.equal(frame.style.properties['--wallpaper-aspect'], '0.5');
  assert.equal(preview.style.top, '-9px');
  assert.equal(image.style.top, '-30px');
});

test('Shift snaps each axis independently and saves the shown position for spare space or overflow', async () => {
  for (const [zoom, roomX, roomY] of [[70, 120, 60], [150, -200, -100]]) {
    const f = createWallpaperFixture({
      spark_wallpaper_enabled: 'true', spark_wallpaper_zoom: String(zoom), spark_wallpaper_pos: '25,75'
    });
    f.idbStore.set('current', 'data:image/png;base64,landscape');
    const frame = f.nodes.get('wallpaper-preview-wrap');
    const preview = f.nodes.get('wallpaper-preview');
    const image = f.nodes.get('wallpaper-image');
    Object.assign(frame, { clientWidth: 400, clientHeight: 200 });
    Object.assign(f.nodes.get('wallpaper-container'), { clientWidth: 1000, clientHeight: 500 });
    for (const node of [preview, image]) Object.assign(node, { naturalWidth: 1000, naturalHeight: 500 });
    await flushWallpaper();

    const key = (type, shiftKey) => f.window.dispatchEvent({ type, key: 'Shift', shiftKey });
    const move = (dx, dy, shiftKey) => frame.listeners.pointermove({
      pointerId: 1, clientX: 200 + roomX / 4 + dx, clientY: 100 - roomY / 4 + dy, shiftKey
    });
    const near = (actual, expected) => assert.ok(Math.abs(Number.parseFloat(actual) - expected) < 1e-6);
    frame.listeners.pointerdown({ button: 0, pointerId: 1, clientX: 200, clientY: 100, preventDefault() {} });
    move(8, 9, false);
    near(preview.style.left, roomX / 2 + 8);
    key('keydown', true);
    near(preview.style.left, roomX / 2);
    near(preview.style.top, roomY / 2 + 9);
    assert.equal(frame.classList.contains('snap-center-x'), true);
    assert.equal(frame.classList.contains('snap-center-y'), false);

    move(-9, 4, true);
    near(preview.style.left, roomX / 2 - 9);
    near(preview.style.top, roomY / 2);
    assert.equal(frame.classList.contains('snap-center-x'), false);
    assert.equal(frame.classList.contains('snap-center-y'), true);

    move(4, -4, true);
    near(image.style.left, roomX * 1.25);
    near(image.style.top, roomY * 1.25);
    assert.equal(frame.classList.contains('snap-center-x'), true);
    assert.equal(frame.classList.contains('snap-center-y'), true);
    key('keyup', false);
    near(preview.style.left, roomX / 2 + 4);
    near(preview.style.top, roomY / 2 - 4);
    key('keydown', true);
    frame.listeners.pointerup({ pointerId: 1, shiftKey: true });
    assert.equal(f.storage.get('spark_wallpaper_pos'), '50,50');
    assert.equal(frame.classList.contains('snap-center-x'), false);
    assert.equal(frame.classList.contains('snap-center-y'), false);
    f.storage.set('spark_wallpaper_pos', '50,50');
    f.window.dispatchEvent({ type: 'storage', key: 'spark_wallpaper_pos' });
    near(preview.style.left, roomX / 2);
    near(preview.style.top, roomY / 2);

    f.storage.set('spark_wallpaper_zoom', '99');
    f.window.dispatchEvent({ type: 'storage', key: 'spark_wallpaper_zoom' });
    f.storage.set('spark_wallpaper_pos', '65,65');
    f.window.dispatchEvent({ type: 'storage', key: 'spark_wallpaper_pos' });
    frame.listeners.pointerdown({ button: 0, pointerId: 1, clientX: 200, clientY: 100, shiftKey: true, preventDefault() {} });
    frame.listeners.pointerup({ pointerId: 1, shiftKey: true });
    assert.equal(f.storage.get('spark_wallpaper_pos'), '65,65');
    frame.listeners.pointerdown({ button: 0, pointerId: 1, clientX: 200, clientY: 100, shiftKey: true, preventDefault() {} });
    frame.listeners.pointermove({ pointerId: 1, clientX: 202, clientY: 101, shiftKey: true });
    assert.equal(frame.classList.contains('snap-center-x'), false);
    assert.equal(frame.classList.contains('snap-center-y'), false);
    frame.listeners.pointerup({ pointerId: 1, shiftKey: true });
    assert.equal(f.storage.get('spark_wallpaper_pos'), '100,100');
  }
});

test('keyboard framing follows the arrows with spare space, overflow, or an exact fit', async () => {
  const f = createWallpaperFixture({ spark_wallpaper_enabled: 'true', spark_wallpaper_zoom: '25' });
  f.idbStore.set('current', 'data:image/png;base64,landscape');
  const frame = f.nodes.get('wallpaper-preview-wrap');
  Object.assign(frame, { clientWidth: 300, clientHeight: 200 });
  Object.assign(f.nodes.get('wallpaper-preview'), { naturalWidth: 1000, naturalHeight: 500 });
  await flushWallpaper();
  const key = key => frame.listeners.keydown({ key, preventDefault() {} });
  key('-');
  assert.equal(f.storage.get('spark_wallpaper_zoom'), '25');
  key('ArrowLeft');
  assert.equal(f.storage.get('spark_wallpaper_pos'), '45,50');
  key('ArrowDown');
  assert.equal(f.storage.get('spark_wallpaper_pos'), '45,55');
  f.storage.set('spark_wallpaper_zoom', '90'); // 360x180 in a 300x200 preview.
  key('ArrowLeft');
  key('ArrowUp');
  assert.equal(f.storage.get('spark_wallpaper_pos'), '50,50');
  f.storage.set('spark_wallpaper_zoom', '100');
  key('ArrowLeft');
  key('ArrowUp');
  assert.equal(f.storage.get('spark_wallpaper_pos'), '55,50');
  f.storage.set('spark_wallpaper_zoom', '295');
  key('+');
  key('+');
  assert.equal(f.storage.get('spark_wallpaper_zoom'), '300');
  assert.equal(f.storage.get('spark_wallpaper_pos'), '55,50');
});
