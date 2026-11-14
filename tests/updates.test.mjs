import assert from 'node:assert/strict';
import { test } from 'node:test';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';
import { createUpdateGate } from '../src/lib/services/update-snapshots.js';
import { createNoteWriter } from '../src/lib/services/note-writer.js';
import { createManifest, parseTag, synchronizeVersion } from '../tools/release/shared.mjs';

test('update replies cannot restore a candidate discarded by a newer channel revision', () => {
  let current;
  const accept = createUpdateGate(state => { current = state; });
  accept({ revision: 2, channel: 'stable', status: 'checking' });
  accept({ revision: 4, channel: 'beta', status: 'idle' });
  assert.equal(accept({ revision: 3, channel: 'stable', status: 'available' }), false);
  assert.equal(accept({ revision: 4, channel: 'beta', status: 'downloading' }), false);
  assert.equal(current.channel, 'beta');
});

test('updater note flush waits for every URL and prevents installation after save failures', async () => {
  let fail = true;
  const saved = [];
  const writer = createNoteWriter({ delay: 60_000, changed() {}, remove: async () => {}, save: async (url, _title, text) => {
    if (fail && url === 'a') throw new Error('disk full');
    saved.push([url, text]);
  } });
  writer.edit('a', 'A', 'pending A'); writer.edit('b', 'B', 'pending B');
  await assert.rejects(writer.flushAll(), /could not be saved/);
  assert.equal(writer.drafts.get('a').text, 'pending A');
  assert.equal(writer.drafts.get('b').status, 'Saved');
  fail = false; await writer.flushAll();
  assert.ok([...writer.drafts.values()].every(draft => draft.status === 'Saved'));
  assert.deepEqual(saved.at(-1), ['a', 'pending A']);
});

test('release tooling synchronizes version files and requires a version-bound manifest signature', () => {
  const root = mkdtempSync(join(tmpdir(), 'spark-release-tools-'));
  try {
    mkdirSync(join(root, 'src-tauri'));
    for (const [path, contents] of [
      ['package.json', '{"version":"0.1.0"}'], ['package-lock.json', '{"version":"0.1.0","packages":{"":{"version":"0.1.0"}}}'],
      ['src-tauri/tauri.conf.json', '{"version":"0.1.0"}'], ['src-tauri/Cargo.toml', '[package]\nname = "spark-browser"\nversion = "0.1.0"\n'],
      ['src-tauri/Cargo.lock', '[[package]]\nname = "spark-browser"\nversion = "0.1.0"\n'],
    ]) writeFileSync(join(root, path), contents);
    const tag = 'v1.1.0-beta.10';
    assert.throws(() => synchronizeVersion(root, tag, false), /do not match/);
    synchronizeVersion(root, tag); synchronizeVersion(root, tag, false);
    assert.equal(JSON.parse(readFileSync(join(root, 'package-lock.json'))).packages[''].version, '1.1.0-beta.10');
    for (const invalid of ['1.0.0', 'v1.0.0\n', 'v1.0.0-beta.01', 'v1.0.0-beta.0', 'v1.0.0-rc.1', 'v1.0.0+test']) assert.throws(() => parseTag(invalid));
    const signature = Buffer.from('untrusted comment: fixture\nabc\ntrusted comment: timestamp:1\tfile:fixture.exe\tversion:1.1.0-beta.10\nxyz').toString('base64');
    const manifest = createManifest(tag, 'SparkBrowser_1.1.0-beta.10_x64-setup.exe', signature, '<script>plain text</script>');
    assert.equal(manifest.platforms['windows-x86_64'].url, `https://github.com/BadTonho/navegador/releases/download/${tag}/SparkBrowser_1.1.0-beta.10_x64-setup.exe`);
    assert.throws(() => createManifest('v1.2.0', 'SparkBrowser_1.2.0_x64-setup.exe', signature, ''), /bound/);
    const capability = JSON.parse(readFileSync('src-tauri/capabilities/default.json', 'utf8'));
    assert.ok(!capability.permissions.some(permission => permission.startsWith('updater:')), 'plugin commands cannot bypass our native release checks');
  } finally {
    assert.equal(dirname(root), resolve(tmpdir()));
    assert.ok(basename(root).startsWith('spark-release-tools-'));
    rmSync(root, { recursive: true, force: true });
  }
});

test('signing setup refuses private-key paths inside the checkout before writing files', () => {
  for (const path of ['.cache/test.key', '..private.key']) {
    const result = spawnSync(process.execPath, ['tools/release/setup-signing.mjs'], {
      encoding: 'utf8', env: { ...process.env, SPARK_SIGNING_KEY_PATH: resolve(path) }, windowsHide: true,
    });
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /outside the repository/);
  }
});
