import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  DEFAULT_VISUAL_PREFERENCES,
  VISUAL_PREFERENCE_STORAGE_KEYS,
  formatAddressForDisplay,
  readVisualPreferences,
  writeVisualPreference,
} from '../src/lib/services/visual-preferences.js';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('visual preferences default off, restore saved booleans and persist edits', () => {
  const values = new Map();
  const storage = {
    getItem: key => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
  };

  assert.deepEqual(readVisualPreferences(storage), DEFAULT_VISUAL_PREFERENCES);
  for (const [name, key] of Object.entries(VISUAL_PREFERENCE_STORAGE_KEYS)) {
    assert.equal(writeVisualPreference(storage, name, true), true);
    assert.equal(values.get(key), 'true');
  }
  assert.deepEqual(readVisualPreferences(storage), {
    wideAddressBar: true,
    showFullUrls: true,
    compactToolbar: true,
    roundedContentFrame: true,
  });
  assert.equal(writeVisualPreference(storage, 'compactToolbar', false), true);
  assert.equal(readVisualPreferences(storage).compactToolbar, false);
});

test('visual preference storage failures and invalid values keep safe defaults', () => {
  const invalid = { getItem: () => 'yes' };
  assert.deepEqual(readVisualPreferences(invalid), DEFAULT_VISUAL_PREFERENCES);
  assert.deepEqual(readVisualPreferences({ getItem: () => { throw new Error('denied'); } }), DEFAULT_VISUAL_PREFERENCES);
  assert.equal(writeVisualPreference({ setItem: () => { throw new Error('denied'); } }, 'wideAddressBar', true), false);
  assert.equal(writeVisualPreference(null, 'wideAddressBar', true), false);
});

test('idle URL formatting hides only HTTP(S) protocol and leading www', () => {
  assert.equal(formatAddressForDisplay('https://www.example.test/path?q=one#part', false), 'example.test/path?q=one#part');
  assert.equal(formatAddressForDisplay('http://example.test:8080/path', false), 'example.test:8080/path');
  assert.equal(formatAddressForDisplay('https://www.example.test/path', true), 'https://www.example.test/path');
  assert.equal(formatAddressForDisplay('spark-internal://new-tab/', false), 'spark-internal://new-tab/');
});

test('appearance controls are searchable and live classes preserve responsive and Zen behavior', () => {
  const appearance = read('src/lib/components/settings/AppearanceSettings.svelte');
  const catalog = read('src/lib/components/settings/settings-catalog.ts');
  const titlebar = read('src/lib/components/titlebar/Titlebar.svelte');
  const actions = read('src/lib/components/titlebar/TitlebarActions.svelte');
  const viewport = read('src/lib/components/viewport/ViewportFrame.svelte');
  for (const id of ['wide-address-bar', 'full-urls', 'compact-toolbar', 'rounded-content-frame']) {
    assert.match(appearance, new RegExp(`id="setting-${id}"`));
    assert.match(catalog, new RegExp(`id: "${id}"`));
  }
  assert.match(titlebar, /--titlebar-height:\s*44px/);
  assert.match(titlebar, /\.density-compact\s*\{[\s\S]*--titlebar-height:\s*40px[\s\S]*--omnibox-height:\s*28px/);
  assert.match(titlebar, /wide-address-bar[\s\S]*max-width:\s*none/);
  assert.match(titlebar, /responsive-compact=\{responsiveCompact\}/);
  assert.match(actions, /var\(--titlebar-control-width/);
  assert.match(viewport, /class:framed=\{\$roundedContentFrame\}/);
  assert.match(viewport, /\.viewport-shell\.zen \.viewport-frame[\s\S]*box-shadow:\s*none/);
});
