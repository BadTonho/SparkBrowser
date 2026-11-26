import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import assert from 'node:assert/strict';
import { test } from 'node:test';
import ts from 'typescript';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

/** Runs the injected media controller against a fake page and asks it for its video addresses. */
async function sources({ videos = [], requests = [], hosts = [], observed = [], key = 'artificial-token' } = {}) {
  const script = read('src-tauri/src/media/controller.js')
    .replaceAll('__MEDIA_TOKEN__', "'artificial-token'")
    .replaceAll('__MEDIA_ENGINE_ECHO__', 'false')
    .replaceAll('__MEDIA_VOLUME__', '100')
    .replaceAll('__PIP_PLAYER__', 'null');
  const context = {
    URL, innerWidth: 1000, innerHeight: 800,
    location: { href: 'https://page.example.test/watch', hostname: 'page.example.test' },
    performance: { getEntriesByType: type => type === 'resource' ? requests.map(name => ({ name })) : [] },
    document: {
      querySelectorAll: selector => selector === 'video' ? videos : selector === '*' ? [...videos, ...hosts] : [], addEventListener() {},
      createElement: () => ({ getContext: () => ({ drawImage(el) { this.drawn = el; } }), toDataURL: () => 'data:image/jpeg;base64,ZnJhbWU=' }),
    },
    PerformanceObserver: class {
      constructor(callback) { this.callback = callback; }
      observe() { this.callback({ getEntries: () => observed.map(name => ({ name })) }); }
    },
    crypto: { randomUUID: () => 'artificial-document' },
    HTMLMediaElement: class { get volume() { return 1; } set volume(_) {} get muted() { return false; } set muted(_) {} },
    MutationObserver: class { observe() {} },
    addEventListener() {},
    setTimeout,
  };
  context.window = context;
  runInNewContext(script, context);
  const result = await context.__sparkMedia(key, { kind: 'sources' });
  return JSON.parse(JSON.stringify(result));
}

const video = ({ currentSrc = '', listed = [], mediaKeys = null, videoHeight = 0, top = 2000, duration = NaN, poster = null, readyState = 0 } = {}) => ({
  localName: 'video', currentSrc, mediaKeys, videoHeight, videoWidth: videoHeight * 2, duration, readyState,
  getAttribute: name => name === 'poster' ? poster : null, querySelectorAll: () => listed,
  getBoundingClientRect: () => ({ left: 0, right: 400, top, bottom: top + 200 }),
});

test('lists direct files and source elements, resolving relative addresses once', async () => {
  const result = await sources({ videos: [video({
    currentSrc: 'https://cdn.example.test/movie.mp4', videoHeight: 720,
    listed: [{ src: 'https://cdn.example.test/movie.mp4', type: 'video/mp4' }, { src: '/movie.webm', type: 'video/webm' }],
  })] });
  assert.equal(result.page, 'https://page.example.test/watch');
  assert.equal(result.protected, false);
  assert.deepEqual(result.sources.map(({ group, ...rest }) => rest), [
    { url: 'https://cdn.example.test/movie.mp4', kind: 'file', height: 720 },
    { url: 'https://page.example.test/movie.webm', kind: 'file', height: 0 },
  ]);
  assert.equal(result.sources[0].group, result.sources[1].group);
});

test('describes each video with a picture and puts the one on the screen first', async () => {
  const result = await sources({ videos: [
    video({ currentSrc: 'https://cdn.example.test/below.mp4', poster: '/below.jpg' }),
    video({ currentSrc: 'https://cdn.example.test/seen.mp4', top: 100, duration: 41.6, readyState: 4, videoHeight: 360 }),
  ] });
  assert.deepEqual(result.sources.map(s => s.url.split('/').at(-1)), ['seen.mp4', 'below.mp4']);
  const [seen, below] = result.videos;
  assert.equal(seen.group, result.sources[0].group);
  assert.deepEqual([seen.area, seen.duration, seen.frame], [80000, 42, 'data:image/jpeg;base64,ZnJhbWU=']);
  assert.deepEqual([below.area, below.duration, below.frame, below.poster], [0, 0, '', 'https://page.example.test/below.jpg']);
});

test('finds the playlist behind a blob player and skips segments and DASH', async () => {
  const result = await sources({
    videos: [video({ currentSrc: 'blob:https://page.example.test/id', listed: [{ src: '/live', type: 'application/vnd.apple.mpegurl' }, { src: '/a.mpd', type: '' }] })],
    requests: ['https://cdn.example.test/hls/master.M3U8?token=1', 'https://cdn.example.test/hls/seg1.ts', 'https://cdn.example.test/app.js'],
  });
  assert.deepEqual(result.sources.map(s => [s.url, s.kind]), [
    ['https://page.example.test/live', 'hls'],
    ['https://cdn.example.test/hls/master.M3U8?token=1', 'hls'],
  ]);
});

test('looks inside web components and reads the playlist stated on the player element', async () => {
  const inner = video({ currentSrc: 'https://cdn.example.test/inside.mp4' });
  const player = {
    localName: 'artificial-player',
    getAttribute: name => name === 'src' ? 'https://cdn.example.test/player/list.m3u8?a=1' : null,
    shadowRoot: { querySelectorAll: () => [inner], querySelector: () => inner },
  };
  const result = await sources({ hosts: [player] });
  assert.deepEqual(result.sources.map(s => s.url), [
    'https://cdn.example.test/inside.mp4',
    'https://cdn.example.test/player/list.m3u8?a=1',
  ]);
  assert.equal(result.sources[0].group, result.sources[1].group, 'the playlist belongs to the video inside its player');
});

test('remembers playlists that no longer fit in the engine resource list', async () => {
  const result = await sources({ observed: ['https://cdn.example.test/late.m3u8', 'https://cdn.example.test/late.js'] });
  assert.deepEqual(result.sources.map(s => s.url), ['https://cdn.example.test/late.m3u8']);
});

test('protected videos are reported and never listed, and a wrong token learns nothing', async () => {
  const videos = [video({ currentSrc: 'https://cdn.example.test/drm.mp4', mediaKeys: {} })];
  const result = await sources({ videos });
  assert.equal(result.protected, true);
  assert.deepEqual(result.sources, []);
  assert.deepEqual(await sources({ videos, key: 'wrong' }), { error: 'media.stale' });
});

test('the button is optional, its popover keeps the page visible and its commands are shell only', () => {
  const store = read('src/lib/stores/titlebarActionsStore.ts');
  assert.ok(store.includes('"qr", "video"'));
  assert.equal(store.match(/DEFAULT_SHOWN_ACTIONS[^\n]*=[^\n]*/)[0].includes('"video"'), false);
  assert.ok(read('src/lib/components/titlebar/TitlebarActions.svelte').includes('data-action="video"'));
  assert.ok(read('src/lib/components/settings/SettingsQuickMenu.svelte').includes('video: "settings.toolbar_video"'));
  assert.ok(!read('src/lib/stores/videoDownloadStore.ts').includes('setOverlayVisibility'));
  assert.match(read('src/lib/services/bridge/overlays.ts'), /nonBlockingOverlayIds = [^\n]*"video-download"/);
  const css = read('src/lib/components/video-download/video-download.css');
  for (const variable of ['--modal-surface-bg', '--modal-surface-border', '--modal-surface-filter', '--modal-surface-shadow']) {
    assert.ok(css.includes(`var(${variable})`), variable);
  }
  assert.ok(read('src/lib/components/panels/PanelSurface.svelte').includes('video-download/video-download.css'));
  for (const command of ['scan_page_videos', 'download_video', 'set_video_download_popover']) {
    assert.ok(read('src-tauri/src/commands/manifest.rs').includes(`"${command}"`));
    assert.ok(read('src-tauri/capabilities/default.json').includes(`"allow-${command.replaceAll('_', '-')}"`));
  }
});

test('a card shows only what can be saved, and the reasons only when nothing is left', () => {
  const exports = {};
  runInNewContext(ts.transpileModule(read('src/lib/services/video-download-options.ts'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText, { exports });
  const source = (video, url, problem = null) => ({ video, url, problem });
  const list = [source('a', 'file'), source('a', 'silent', 'video.separate_audio'),
    source('b', 'one', 'video.separate_audio'), source('b', 'two', 'video.separate_audio'), source('b', 'three', 'video.live')];
  const card = id => JSON.parse(JSON.stringify(exports.videoOptions(list, id)));
  assert.deepEqual(card('a'), { options: [list[0]], reasons: [] });
  assert.deepEqual(card('b'), { options: [], reasons: ['video.separate_audio', 'video.live'] });
});

test('every problem code the backend sends has a message in both languages', () => {
  const panel = read('src/lib/components/video-download/VideoDownloadPanel.svelte');
  const backend = ['src-tauri/src/video_download/mod.rs', 'src-tauri/src/video_download/playlist.rs', 'src-tauri/src/video_download/preview.rs', 'src-tauri/src/downloads/hls.rs']
    .map(path => read(path).split('#[cfg(test)]')[0]).join('\n');
  for (const [, code] of backend.matchAll(/"(video\.[a-z_]+)"/g)) assert.ok(panel.includes(`"${code}"`), code);
  // File names such as `video.ts` appear in the Rust tests, not as codes.
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/downloads.json`));
    for (const [key] of panel.matchAll(/video_download\.[a-z_]+/g)) assert.ok(catalog[key], `${language}: ${key}`);
  }
});
