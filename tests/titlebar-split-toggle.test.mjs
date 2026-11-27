import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(path, 'utf8');
const titlebar = read('src/lib/components/titlebar/Titlebar.svelte');

test('horizontal tabs get a split toggle in the left corner of the address row', () => {
  const left = /<div class="titlebar-left">([\s\S]*?)<!-- Centered Omnibox/.exec(titlebar)?.[1];
  assert.ok(left, 'the left side of the address row is found');
  // First thing in the corner, before the private badge and the workspace switcher.
  assert.match(left, /^\s*\{#if \$tabLayout === "horizontal"\}\s*(<!--[^>]*-->\s*)?<button\s+class="nav-btn split-btn"/);
  assert.ok(left.indexOf('titlebar-split-toggle') < left.indexOf('<PrivateBadge />'));
  // Vertical tabs already have their split buttons in the tab list.
  const navigation = /<div class="nav-controls">([\s\S]*?)<\/div>\s*<div class="omnibox-wrapper"/.exec(titlebar)?.[1];
  assert.ok(!navigation.includes('titlebar-split-toggle'), 'it left the navigation buttons');
});

test('the toggle opens or closes the split and shows its shortcut', () => {
  assert.match(titlebar, /import \{ enterSplitView, exitSplitView \} from "\.\.\/\.\.\/stores\/splitViewActions";/);
  assert.match(titlebar, /onclick=\{\(\) => void \(splitHorizontal \? exitSplitView\(\) : enterSplitView\(\)\)\}/);
  assert.match(titlebar, /aria-pressed=\{splitHorizontal\}/);
  assert.match(titlebar, /aria-label=\{\$t\(splitHorizontal \? "tabs\.split_exit" : "tabs\.split_enter"\)\}/);
  assert.match(titlebar, /shortcutLabel\("split-view"\)/);
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/tabs.json`));
    assert.ok(catalog['tabs.split_enter'] && catalog['tabs.split_exit'], language);
  }
});

test('the split-view shortcut is defined, handled and accepted by the native side and backups', () => {
  assert.match(read('src/lib/stores/shortcutsStore.ts'), /\{ id: "split-view", label: "Open or close split view"[^\n]*defaults: \[combo\("S", \{ ctrl: true, shift: true \}\)\] \}/);
  assert.match(read('src/App.svelte'), /case "split-view": void \(get\(splitLayout\) \? exitSplitView\(\) : enterSplitView\(\)\); break;/);
  // Pressed inside a page, the shortcut reaches the shell only for actions the bridge accepts.
  assert.match(read('src-tauri/src/window/keyboard_bridge.rs'), /\| "split-view"/);
  for (const language of ['en', 'pt-BR']) {
    const settings = JSON.parse(read(`src/lib/i18n/catalogs/${language}/settings.json`));
    assert.ok(settings['settings.open_or_close_split_view_4d1a'] && settings['settings.split_window_between_two_tabs_7c3e'], language);
  }
  assert.equal(JSON.parse(read('src/lib/i18n/catalogs/en/settings.json'))['settings.open_or_close_split_view_4d1a'], 'Open or close split view', 'the label is translated by its English text');
});
