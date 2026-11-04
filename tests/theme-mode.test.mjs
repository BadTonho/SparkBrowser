import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  createThemePreference,
  readThemeMode,
  THEME_MODES,
  THEME_MODE_STORAGE_KEY,
} from '../src/lib/services/theme-preference.js';

test('theme mode defaults to dark for missing, empty, or invalid storage', () => {
  assert.deepEqual(THEME_MODES, ['dark', 'light']);

  for (const value of [null, undefined, '', 'invalid', 'DARK', 'LIGHT', 'system', '123']) {
    assert.equal(readThemeMode({ getItem: () => value }), 'dark');
  }

  // Denied storage should fallback to dark
  assert.equal(
    readThemeMode({
      getItem: () => {
        throw new Error('denied');
      },
    }),
    'dark'
  );

  // Literal light should be read correctly
  assert.equal(readThemeMode({ getItem: () => 'light' }), 'light');
});

test('createThemePreference initializes, sets root attribute, and persists mode changes', () => {
  const store = new Map();
  const attributes = new Map();
  const changes = [];

  const mockStorage = {
    getItem: (k) => store.get(k) ?? null,
    setItem: (k, v) => store.set(k, v),
  };

  const mockRoot = {
    setAttribute: (k, v) => attributes.set(k, v),
  };

  const pref = createThemePreference({
    storage: mockStorage,
    root: mockRoot,
    changed: (mode) => changes.push(mode),
  });

  // Default initial is dark
  assert.equal(pref.get(), 'dark');
  assert.equal(attributes.get('data-theme'), 'dark');

  // Switch to light
  assert.equal(pref.set('light'), 'light');
  assert.equal(pref.get(), 'light');
  assert.equal(attributes.get('data-theme'), 'light');
  assert.equal(store.get(THEME_MODE_STORAGE_KEY), 'light');
  assert.deepEqual(changes, ['light']);

  // Invalid mode is ignored
  assert.equal(pref.set('unsupported'), 'light');
  assert.equal(pref.get(), 'light');
  assert.equal(attributes.get('data-theme'), 'light');
  assert.equal(store.get(THEME_MODE_STORAGE_KEY), 'light');

  // Switch back to dark
  assert.equal(pref.set('dark'), 'dark');
  assert.equal(pref.get(), 'dark');
  assert.equal(attributes.get('data-theme'), 'dark');
  assert.equal(store.get(THEME_MODE_STORAGE_KEY), 'dark');
  assert.deepEqual(changes, ['light', 'dark']);
});

test('createThemePreference survives when storage is unavailable', () => {
  const attributes = new Map();
  const pref = createThemePreference({
    storage: {
      getItem: () => {
        throw new Error('QuotaExceeded');
      },
      setItem: () => {
        throw new Error('QuotaExceeded');
      },
    },
    root: {
      setAttribute: (k, v) => attributes.set(k, v),
    },
  });

  assert.equal(pref.get(), 'dark');
  assert.equal(attributes.get('data-theme'), 'dark');

  // Should successfully change in-memory and on root even if setItem throws
  assert.equal(pref.set('light'), 'light');
  assert.equal(pref.get(), 'light');
  assert.equal(attributes.get('data-theme'), 'light');
});
