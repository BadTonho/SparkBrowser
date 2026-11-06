import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import assert from 'node:assert/strict';
import { test } from 'node:test';

test('ordinary document mutations do not schedule media scans at default volume', async () => {
  let mutation, timers = 0, scans = 0;
  class Media {}
  Object.defineProperties(Media.prototype, {
    volume: { get() { return 1; }, set() {} },
    muted: { get() { return false; }, set() {} },
  });
  const document = {
    title: 'Fixture', pictureInPictureElement: null, pictureInPictureEnabled: true,
    querySelectorAll() { scans++; return []; }, addEventListener() {},
  };
  const window = { addEventListener() {} };
  const source = readFileSync('src-tauri/src/media/controller.js', 'utf8')
    .replace('__MEDIA_TOKEN__', JSON.stringify('fixture'))
    .replace('__MEDIA_VOLUME__', '100')
    .replace('__PIP_PLAYER__', '(() => {})');
  runInNewContext(source, {
    window, document, HTMLMediaElement: Media, crypto: { randomUUID: () => 'document' },
    MutationObserver: class { constructor(callback) { mutation = callback; } observe() {} },
    navigator: { userActivation: { isActive: false } },
    setTimeout() { timers++; },
  });
  mutation();
  assert.equal(timers, 0);
  assert.equal(scans, 0);
  await window.__veloceMedia('fixture', { kind: 'volume', value: 50 });
  const previous = timers;
  mutation();
  assert.equal(timers, previous + 1, 'non-default volume still tracks newly inserted media');
});
