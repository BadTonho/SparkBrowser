import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(path, 'utf8');
const titlebar = read('src/lib/components/titlebar/Titlebar.svelte');

test('horizontal tabs get a split toggle at the left of the navigation buttons', () => {
  const controls = /<div class="nav-controls">([\s\S]*?)<\/div>\s*<div class="omnibox-wrapper"/.exec(titlebar)?.[1];
  assert.ok(controls, 'the navigation buttons are found');
  const toggle = controls.indexOf('data-testid="titlebar-split-toggle"');
  const back = controls.indexOf('triggerBack');
  assert.ok(toggle >= 0 && back > toggle, 'the toggle comes before Back');
  // Vertical tabs already have their split buttons in the tab list.
  assert.match(controls.slice(0, toggle), /\{#if \$tabLayout === "horizontal"\}\s*(<!--[^>]*-->\s*)?<button\s+class="nav-btn split-btn"/);
});

test('the toggle opens or closes the split and says which one it does', () => {
  assert.match(titlebar, /import \{ enterSplitView, exitSplitView \} from "\.\.\/\.\.\/stores\/splitViewActions";/);
  assert.match(titlebar, /onclick=\{\(\) => void \(splitHorizontal \? exitSplitView\(\) : enterSplitView\(\)\)\}/);
  assert.match(titlebar, /aria-pressed=\{splitHorizontal\}/);
  assert.match(titlebar, /aria-label=\{\$t\(splitHorizontal \? "tabs\.split_exit" : "tabs\.split_enter"\)\}/);
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/tabs.json`));
    assert.ok(catalog['tabs.split_enter'] && catalog['tabs.split_exit'], language);
  }
});
