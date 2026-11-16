import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

const EMBED = 'https://www.youtube-nocookie.com';
function fixture(storage = {}) {
  const nodes = new Map(), events = new Map(), dispatched = [], intervals = [], posts = [], store = new Map(Object.entries(storage));
  const node = id => {
    if (!nodes.has(id)) nodes.set(id, {id, hidden: false, disabled: false, textContent: '', checked: false, value: '', listeners: {}, attrs: {},
      addEventListener(name, fn) { this.listeners[name] = fn; },
      setAttribute(name, value) { this.attrs[name] = value; },
      getAttribute(name) { return this.attrs[name]; },
      removeAttribute(name) { delete this.attrs[name]; delete this[name]; },
      style: {setProperty(name, value) { this[name] = value; }, removeProperty(name) { delete this[name]; }},
      offsetWidth: 0, clientWidth: 0, clientHeight: 0,
      focus() { this.focused = true; }});
    return nodes.get(id);
  };
  const frameWindow = {postMessage: (message, origin) => posts.push({message: JSON.parse(message), origin})};
  node('youtube-frame').contentWindow = frameWindow;
  const window = {
    addEventListener: (name, fn) => events.set(name, fn),
    dispatchEvent: event => dispatched.push(event),
    setInterval: fn => { intervals.push(fn); return intervals.length; },
    clearInterval: id => { intervals[id - 1] = null; },
    location: {origin: 'http://tauri.localhost'},
  };
  const localStorage = {getItem: key => store.get(key) ?? null, setItem: (key, value) => store.set(key, value)};
  class CustomEvent { constructor(type, options) { this.type = type; this.detail = options.detail; } }
  const document = {hidden: false, getElementById: node, addEventListener: (name, fn) => events.set(`document:${name}`, fn)};
  runInNewContext(readFileSync('public/new-tab/media-card.js', 'utf8'), {window, document, localStorage, String, Date, Math, Number, JSON});
  runInNewContext(readFileSync('public/new-tab/youtube.js', 'utf8'), {window, document, localStorage, CustomEvent, String, Date, Math, Number, JSON, encodeURIComponent});
  const labels = JSON.parse(readFileSync('src/lib/i18n/catalogs/en/newtab.json', 'utf8'));
  events.get('spark-new-tab')({detail: {labels, private: false}});
  return {node, window, document, dispatched, events, intervals, posts, store};
}
const video = {tab_id: 't', source: 'youtube', video_id: 'dQw4w9WgXcQ', title: 'A <b>song</b>', playing: true, image: null};
const jpeg = 'data:image/jpeg;base64,/9j/AAA=';

test('the card renders playback as text, toggles icons and hides when disabled', () => {
  const f = fixture();
  f.window.__sparkYoutubeRender({youtube_enabled: false}, video, false);
  assert.equal(f.node('youtube-widget').hidden, true);
  f.window.__sparkYoutubeRender({youtube_enabled: true}, video, false);
  assert.equal(f.node('youtube-widget').hidden, false);
  assert.equal(f.node('youtube-title').textContent, 'A <b>song</b>');
  assert.equal(f.node('youtube-heading').textContent, 'YouTube');
  assert.equal(f.node('youtube-logo-video').hidden, false);
  assert.equal(f.node('youtube-toggle').textContent, '⏸');
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, source: 'music', playing: false}, false);
  assert.equal(f.node('youtube-heading').textContent, 'YouTube Music');
  assert.equal(f.node('youtube-logo-music').hidden, false);
  assert.equal(f.node('youtube-logo-video').hidden, true);
  assert.equal(f.node('youtube-toggle').textContent, '▶');
  f.window.__sparkYoutubeRender({youtube_enabled: true}, null, false);
  assert.equal(f.node('youtube-controls').hidden, true);
  assert.match(f.node('youtube-status').textContent, /YouTube/);
});

test('the logo is the only way to the tab: enabled with a video, inert without one', () => {
  const f = fixture();
  f.window.__sparkYoutubeRender({youtube_enabled: true}, video, false);
  assert.equal(f.node('youtube-open').disabled, false);
  assert.match(f.node('youtube-open').attrs['aria-label'], /Go to tab/);
  f.node('youtube-open').listeners.click();
  assert.equal(f.dispatched.at(-1).detail.operation, 'youtube_open');
  f.window.__sparkYoutubeRender({youtube_enabled: true}, null, false);
  assert.equal(f.node('youtube-open').disabled, true);
  const html = readFileSync('public/new-tab.html', 'utf8');
  assert.ok(!/id="youtube-open"[^>]*icon-button/.test(html), 'no separate arrow button');
});

test('the progress bar shows the video length, advances while playing and hides for live streams', () => {
  const f = fixture();
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, position: 65, duration: 3725}, false);
  assert.equal(f.node('youtube-progress').hidden, false);
  assert.equal(f.node('youtube-progress').max, 3725);
  assert.equal(f.node('youtube-elapsed').textContent, '1:05');
  assert.equal(f.node('youtube-total').textContent, '1:02:05');
  assert.equal(f.intervals.filter(Boolean).length, 1, 'ticks only while playing');
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, playing: false, position: 10, duration: 100}, false);
  assert.equal(f.intervals.filter(Boolean).length, 0);
  assert.equal(f.node('youtube-elapsed').textContent, '0:10');
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, position: 10, duration: null}, false);
  assert.equal(f.node('youtube-progress').hidden, true);
  assert.equal(f.node('youtube-times').hidden, true);
});

test('a title wider than its box scrolls from side to side and a short one stays still', () => {
  const f = fixture();
  f.node('youtube-title').offsetWidth = 400; f.node('youtube-marquee').clientWidth = 150;
  f.window.__sparkYoutubeRender({youtube_enabled: true}, video, false);
  assert.equal(f.node('youtube-marquee').attrs['data-scrolling'], 'true');
  assert.equal(f.node('youtube-marquee').style['--shift'], '-250px');
  f.node('youtube-title').offsetWidth = 100;
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, title: 'Short'}, false);
  assert.equal(f.node('youtube-marquee').attrs['data-scrolling'], undefined);
  const css = readFileSync('public/new-tab/media-card.css', 'utf8');
  assert.match(css, /alternate/);
  assert.match(css, /prefers-reduced-motion/);
});

test('only JPEG data URLs are used as thumbnails', () => {
  const f = fixture();
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, image: 'https://evil.test/a.jpg'}, false);
  assert.equal(f.node('youtube-cover').hidden, true);
  assert.equal(f.node('youtube-widget').attrs['data-media'], 'false');
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, image: 'data:image/svg+xml;base64,AAAA'}, false);
  assert.equal(f.node('youtube-cover').hidden, true);
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, image: jpeg}, false);
  assert.equal(f.node('youtube-cover').hidden, false);
  assert.equal(f.node('youtube-widget').attrs['data-media'], 'true');
});

test('the thumbnail style and the preview kind are local choices that default to small and still', () => {
  const f = fixture();
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, image: jpeg}, false);
  assert.equal(f.node('youtube-widget').attrs['data-thumb'], 'small');
  assert.equal(f.node('youtube-widget').attrs['data-preview'], 'image');
  assert.equal(f.node('youtube-frame').hidden, true, 'no player unless the video preview is chosen');
  f.node('youtube-thumb').listeners.change({target: {value: 'cover'}});
  assert.equal(f.store.get('spark_youtube_thumbnail'), 'cover');
  assert.equal(f.node('youtube-widget').attrs['data-thumb'], 'cover');
  f.node('youtube-thumb').listeners.change({target: {value: '<script>'}});
  assert.equal(f.node('youtube-widget').attrs['data-thumb'], 'small', 'unknown values fall back');
  const restored = fixture({spark_youtube_thumbnail: 'cover', spark_youtube_preview: 'video'});
  restored.window.__sparkYoutubeRender({youtube_enabled: true}, video, false);
  assert.equal(restored.node('youtube-widget').attrs['data-thumb'], 'cover');
  assert.equal(restored.node('youtube-widget').attrs['data-preview'], 'video');
});

test('the muted preview embeds only the privacy-enhanced player for a valid ID and follows the tab', () => {
  const f = fixture({spark_youtube_preview: 'video'});
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, position: 30, duration: 200}, false);
  const frame = f.node('youtube-frame');
  assert.equal(frame.hidden, false);
  assert.match(frame.src, new RegExp(`^${EMBED}/embed/dQw4w9WgXcQ\\?`));
  assert.match(frame.src, /mute=1/);
  assert.match(frame.src, /controls=0/);
  assert.match(frame.src, /autoplay=1/);
  assert.match(frame.src, /start=30/);
  const source = frame.src;
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, position: 31, duration: 200}, false);
  assert.equal(frame.src, source, 'the same video keeps its player');
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, playing: false, position: 31, duration: 200}, false);
  assert.deepEqual(f.posts.at(-1).message.func, 'pauseVideo');
  assert.equal(f.posts.at(-1).origin, EMBED);
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, position: 31, duration: 200}, false);
  assert.equal(f.posts.at(-1).message.func, 'playVideo');
  // A seek in the tab is followed once the embedded clock is far enough away.
  const message = data => f.events.get('message')({origin: EMBED, source: frame.contentWindow, data: JSON.stringify(data)});
  message({event: 'infoDelivery', info: {currentTime: 31}});
  const before = f.posts.length;
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, position: 120, duration: 200}, false);
  message({event: 'infoDelivery', info: {currentTime: 33}});
  assert.equal(f.posts.length, before + 1);
  assert.equal(f.posts.at(-1).message.func, 'seekTo');
  assert.ok(Math.abs(f.posts.at(-1).message.args[0] - 120) < 2);
  // Messages from anywhere else are ignored.
  const count = f.posts.length;
  f.events.get('message')({origin: 'https://evil.test', source: frame.contentWindow, data: JSON.stringify({event: 'infoDelivery', info: {currentTime: 0}})});
  f.events.get('message')({origin: EMBED, source: {}, data: '{"event":"infoDelivery","info":{"currentTime":0}}'});
  assert.equal(f.posts.length, count);
});

test('the player is removed for bad IDs, other preview kinds, hidden pages and disabled widgets', () => {
  const f = fixture({spark_youtube_preview: 'video'});
  const frame = f.node('youtube-frame');
  f.window.__sparkYoutubeRender({youtube_enabled: true}, {...video, video_id: '../../evil'}, false);
  assert.equal(frame.hidden, true);
  assert.equal(frame.src, undefined);
  f.window.__sparkYoutubeRender({youtube_enabled: true}, video, false);
  assert.equal(frame.hidden, false);
  f.document.hidden = true;
  f.events.get('document:visibilitychange')();
  assert.equal(frame.hidden, true);
  assert.equal(frame.attrs.src, undefined);
  f.document.hidden = false;
  f.window.__sparkYoutubeRender({youtube_enabled: true}, video, false);
  assert.equal(frame.hidden, false);
  f.node('youtube-preview').listeners.change({target: {value: 'image'}});
  assert.equal(frame.hidden, true);
  f.node('youtube-preview').listeners.change({target: {value: 'video'}});
  f.window.__sparkYoutubeRender({youtube_enabled: false}, video, false);
  assert.equal(frame.hidden, true);
});

test('the player is scaled to cover the whole widget only in cover mode', () => {
  const f = fixture({spark_youtube_preview: 'video', spark_youtube_thumbnail: 'cover'});
  f.node('youtube-widget').clientWidth = 320; f.node('youtube-widget').clientHeight = 300;
  f.window.__sparkYoutubeRender({youtube_enabled: true}, video, false);
  assert.equal(f.node('youtube-frame').style.height, '300px');
  assert.equal(f.node('youtube-frame').style.width, `${Math.ceil(300 * 16 / 9)}px`);
  f.node('youtube-thumb').listeners.change({target: {value: 'small'}});
  assert.equal(f.node('youtube-frame').style.width, undefined);
});

test('controls send native operations and the setting is emitted as a widget preference', () => {
  const f = fixture();
  for (const [id, operation] of [['youtube-previous', 'youtube_previous'], ['youtube-toggle', 'youtube_toggle'], ['youtube-next', 'youtube_next'], ['youtube-open', 'youtube_open']]) {
    f.node(id).listeners.click();
    assert.equal(f.dispatched.at(-1).detail.operation, operation);
  }
  f.node('youtube-enabled').listeners.change({target: {checked: true}});
  assert.equal(f.dispatched.at(-1).type, 'spark-youtube-setting');
  assert.equal(f.dispatched.at(-1).detail.youtube_enabled, true);
});

test('the media category switches between Spotify and YouTube panels with arrow keys', () => {
  const f = fixture();
  f.node('media-tab-youtube').listeners.click();
  assert.equal(f.node('youtube-panel').hidden, false);
  assert.equal(f.node('spotify-panel').hidden, true);
  assert.equal(f.node('media-tab-youtube').attrs['aria-selected'], 'true');
  f.node('media-tab-youtube').listeners.keydown({key: 'ArrowRight'});
  assert.equal(f.node('spotify-panel').hidden, false);
  assert.equal(f.node('media-tab-spotify').focused, true);
});

test('the new-tab page wires the YouTube assets, the grouped menu entry, private hiding and a narrow frame policy', () => {
  const html = readFileSync('public/new-tab.html', 'utf8');
  assert.match(html, /\/new-tab\/youtube\.js/);
  assert.match(html, /\/new-tab\/youtube\.css/);
  assert.match(html, /id="menu-item-spotify"[\s\S]*?data-label="mediaCategory"/);
  assert.match(html, /frame-src https:\/\/www\.youtube-nocookie\.com;/);
  assert.match(html, /connect-src 'none'/);
  assert.match(html, /<iframe id="youtube-frame"[^>]*sandbox="allow-scripts allow-same-origin"/);
  assert.match(readFileSync('public/new-tab/youtube.css', 'utf8'), /data-private="true"[^{]*#youtube-widget/);
  const widgets = readFileSync('public/new-tab/widgets.js', 'utf8');
  assert.match(widgets, /__sparkYoutubeRender/);
  assert.match(widgets, /spark-youtube-setting/);
});
