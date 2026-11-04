import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { webcrypto as crypto } from 'node:crypto';

test('activity reports only form/media flags, deduplicates and releases protection on reset/pause', () => {
  const listeners = new Map();
  const timers = [];
  const media = { paused: true, ended: false };
  const window = { location: { href: '' } }; window.top = window;
  const document = { readyState: 'complete', querySelectorAll: () => [media], addEventListener: (name, fn) => listeners.set(name, fn) };
  class Element { value = 'private form contents'; matches() { return true; } }
  const source = readFileSync(new URL('../src-tauri/src/window/page_activity.js', import.meta.url), 'utf8').replace('__ACTIVITY_TOKEN__', '"test-generation"');
  vm.runInNewContext(source, { window, document, Element, URLSearchParams, crypto, setTimeout: fn => timers.push(fn) });
  const fire = name => { listeners.get(name)({ target: new Element() }); timers.splice(0).forEach(fn => fn()); return new URL(window.location.href); };
  const first = fire('input');
  assert.equal(first.searchParams.get('dirty'), 'true');
  assert.match(first.searchParams.get('document'), /^[a-f0-9]{32}$/);
  assert.equal(first.searchParams.get('sequence'), '1');
  assert.equal(fire('input').href, first.href, 'unchanged activity is deduplicated');
  assert.ok(!window.location.href.includes('private'));
  const reset = fire('reset');
  assert.equal(reset.searchParams.get('dirty'), 'false');
  assert.equal(reset.searchParams.get('document'), first.searchParams.get('document'));
  assert.equal(reset.searchParams.get('sequence'), '2');
  media.paused = false;
  assert.equal(fire('play').searchParams.get('media'), 'true');
  media.paused = true;
  assert.equal(fire('pause').searchParams.get('media'), 'false');
});
