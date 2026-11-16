import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

const read = path => readFileSync(path, 'utf8');
function load(path) {
  const exports = {};
  runInNewContext(ts.transpileModule(read(path), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText, { exports, Number, Math, String });
  return exports;
}
const { formatClock, elapsedSeconds } = load('src/lib/services/media-time.ts');

test('clock formatting uses m:ss, h:mm:ss and tolerates bad numbers', () => {
  assert.equal(formatClock(0), '0:00');
  assert.equal(formatClock(65.9), '1:05');
  assert.equal(formatClock(3725), '1:02:05');
  assert.equal(formatClock(-4), '0:00');
  assert.equal(formatClock(NaN), '0:00');
});

test('elapsed time only advances while playing and never passes the video length', () => {
  assert.equal(elapsedSeconds(10, 100, true, 1000, 4000), 13);
  assert.equal(elapsedSeconds(10, 100, false, 1000, 4000), 10);
  assert.equal(elapsedSeconds(99, 100, true, 0, 60000), 100);
  assert.equal(elapsedSeconds(10, 100, true, 5000, 1000), 10, 'a clock that goes backwards is ignored');
});

test('the vertical-tabs sidebar renders the card at the bottom through the backend commands and event', () => {
  const sidebar = read('src/lib/components/sidebar/Sidebar.svelte');
  assert.match(sidebar, /import NowPlaying from "\.\/NowPlaying\.svelte"/);
  assert.ok(sidebar.indexOf('<TabList />') < sidebar.indexOf('<NowPlaying />'), 'card comes after the tab list');
  const bridge = read('src/lib/services/bridge/now-playing.ts');
  assert.match(bridge, /"get_now_playing"/);
  assert.match(bridge, /"control_now_playing"/);
  assert.match(bridge, /"youtube-now-playing"/);
  const component = read('src/lib/components/sidebar/NowPlaying.svelte');
  for (const operation of ['toggle', 'next', 'previous', 'open']) assert.match(component, new RegExp(`run\\("${operation}"\\)`));
  assert.match(component, /spark_youtube_thumbnail/);
  assert.match(component, /prefers-reduced-motion/);
  assert.match(component, /:global\(\[data-scrolling="true"\]\)/, 'the marquee state is set by an action, so Svelte must not drop its CSS');
});

test('the backend commands are registered, listed in the ACL manifest and allowed for the shell', () => {
  const manifest = read('src-tauri/src/commands/manifest.rs');
  const lib = read('src-tauri/src/lib.rs');
  const capability = read('src-tauri/capabilities/default.json');
  for (const [command, permission] of [['get_now_playing', 'allow-get-now-playing'], ['control_now_playing', 'allow-control-now-playing']]) {
    assert.match(manifest, new RegExp(`"${command}"`));
    assert.match(lib, new RegExp(`\\b${command},`));
    assert.match(capability, new RegExp(`"${permission}"`));
  }
});

test('catalogs, the new-tab option and the docs cover the sidebar card', () => {
  for (const language of ['en', 'pt-BR']) {
    const sidebar = JSON.parse(read(`src/lib/i18n/catalogs/${language}/sidebar.json`));
    for (const key of ['region', 'open', 'previous', 'next', 'play', 'pause', 'progress']) assert.ok(sidebar[`sidebar.now_playing_${key}`], `${language} ${key}`);
    assert.ok(JSON.parse(read(`src/lib/i18n/catalogs/${language}/newtab.json`))['newtab.widgets.youtubeSidebar']);
  }
  assert.match(read('public/new-tab.html'), /id="youtube-sidebar" type="checkbox"/);
  assert.match(read('docs/en/features/youtube-widget.md'), /vertical tabs/i);
  assert.match(read('docs/pt-BR/features/youtube-widget.md'), /guias verticais/i);
});
