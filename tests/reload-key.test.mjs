import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { isReloadKey } from '../src/lib/services/reload-key.js';

const key = (value, modifiers = {}) => ({ key: value, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, ...modifiers });

test('F5 and Ctrl+Shift+R reload the page, other keys do not', () => {
  assert.equal(isReloadKey(key('F5')), true);
  assert.equal(isReloadKey(key('F5', { ctrlKey: true })), true);
  assert.equal(isReloadKey(key('F5', { shiftKey: true })), true);
  assert.equal(isReloadKey(key('R', { ctrlKey: true, shiftKey: true })), true);
  assert.equal(isReloadKey(key('r', { ctrlKey: true, shiftKey: true })), true);
  assert.equal(isReloadKey(key('F5', { altKey: true })), false);
  assert.equal(isReloadKey(key('r', { ctrlKey: true })), false);
  assert.equal(isReloadKey(key('F6')), false);
});

test('the interface never reloads itself: its key handler maps F5 to the reload-tab action', () => {
  const app = readFileSync('src/App.svelte', 'utf8');
  assert.match(app, /isReloadKey\(event\)\) \{\s*event\.preventDefault\(\);\s*if \(!event\.repeat\) dispatchShortcut\("reload-tab"/);
});
