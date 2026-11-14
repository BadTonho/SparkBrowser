import assert from 'node:assert/strict';
import test from 'node:test';
import { migrateLegacyKeys } from '../src/lib/services/storage.js';
import { signingKeyPath } from '../tools/release/shared.mjs';

/** The small part of the Storage API the migration uses. */
function fakeStorage(initial = {}, { failWrites = false } = {}) {
  const map = new Map(Object.entries(initial));
  return {
    map,
    get length() { return map.size; },
    key: (index) => [...map.keys()][index] ?? null,
    getItem: (key) => (map.has(key) ? map.get(key) : null),
    setItem(key, value) { if (failWrites) throw new Error('quota'); map.set(key, String(value)); },
    removeItem: (key) => { map.delete(key); },
  };
}

test('old veloce_* settings move to spark_* and the old keys disappear', () => {
  const storage = fakeStorage({ veloce_theme_mode: 'light', veloce_resident_limit: '8', unrelated: 'x', spark_language: 'en' });
  assert.equal(migrateLegacyKeys(storage), 2);
  assert.deepEqual(Object.fromEntries(storage.map), { unrelated: 'x', spark_language: 'en', spark_theme_mode: 'light', spark_resident_limit: '8' });
});

test('a value the new key already holds is never overwritten by an old one', () => {
  const storage = fakeStorage({ veloce_theme_mode: 'light', spark_theme_mode: 'dark' });
  migrateLegacyKeys(storage);
  assert.equal(storage.getItem('spark_theme_mode'), 'dark');
  assert.equal(storage.getItem('veloce_theme_mode'), null);
});

test('migrating is idempotent and leaves old keys in place when storage cannot be written', () => {
  const storage = fakeStorage({ veloce_a: '1' });
  assert.equal(migrateLegacyKeys(storage), 1);
  assert.equal(migrateLegacyKeys(storage), 0);
  const broken = fakeStorage({ veloce_a: '1' }, { failWrites: true });
  assert.equal(migrateLegacyKeys(broken), 0);
  assert.equal(broken.getItem('veloce_a'), '1', 'nothing is lost when the write fails');
});

test('the release signing key is found where it already is, so the rename cannot strand it', () => {
  const home = '/home/dev';
  const at = (...paths) => (path) => paths.some((p) => path.replaceAll('\\', '/').endsWith(p));
  const legacyOnly = signingKeyPath({}, home, at('.veloce-signing/updater.key'));
  assert.match(legacyOnly.replaceAll('\\', '/'), /\.veloce-signing\/updater\.key$/);
  const both = signingKeyPath({}, home, at('.veloce-signing/updater.key', '.spark-signing/updater.key'));
  assert.match(both.replaceAll('\\', '/'), /\.spark-signing\/updater\.key$/);
  const neither = signingKeyPath({}, home, at());
  assert.match(neither.replaceAll('\\', '/'), /\.spark-signing\/updater\.key$/);
  assert.match(signingKeyPath({ SPARK_SIGNING_KEY_PATH: '/keys/new.key' }, home, at()).replaceAll('\\', '/'), /\/keys\/new\.key$/);
  assert.match(signingKeyPath({ VELOCE_SIGNING_KEY_PATH: '/keys/old.key' }, home, at()).replaceAll('\\', '/'), /\/keys\/old\.key$/);
  assert.match(signingKeyPath({ SPARK_SIGNING_KEY_PATH: '/keys/new.key', VELOCE_SIGNING_KEY_PATH: '/keys/old.key' }, home, at()).replaceAll('\\', '/'), /\/keys\/new\.key$/);
});
