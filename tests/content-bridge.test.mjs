import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import { test } from 'node:test';
import assert from 'node:assert/strict';

function bridge(native) {
  const timers = [], messages = [];
  const window = { location: { href: 'https://fixture.test/original' } }; window.top = window;
  vm.runInNewContext(readFileSync('src-tauri/src/window/content_bridge.js', 'utf8').replace('__CONTENT_NATIVE__', String(native)), { window, setTimeout: callback => { timers.push(callback); return timers.length; } });
  return { window, messages, flush: () => timers.splice(0).forEach(callback => callback()) };
}

test('passive content reports wait for the document binding and never navigate while it installs', () => {
  const f = bridge(true);
  f.window.__velocePostContent('veloce-internal://new-tab/read?sequence=1');
  f.window.__velocePostContent('veloce-internal://page-activity/state?sequence=2');
  assert.equal(f.window.location.href, 'https://fixture.test/original');
  f.window.__veloceContentSignal = message => f.messages.push(JSON.parse(message));
  f.flush();
  assert.deepEqual(f.messages.map(message => message.veloceContent), [
    'veloce-internal://new-tab/read?sequence=1', 'veloce-internal://page-activity/state?sequence=2',
  ]);
  assert.equal(f.window.location.href, 'https://fixture.test/original');
});

test('legacy engines retain the existing authenticated navigation transport', () => {
  const f = bridge(false);
  f.window.__velocePostContent('veloce-internal://new-tab/read?token=fixture');
  assert.equal(f.window.location.href, 'veloce-internal://new-tab/read?token=fixture');
});
