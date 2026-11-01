import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import { test } from 'node:test';
import assert from 'node:assert/strict';

test('activity reports only form/media flags, deduplicates and releases protection on reset/pause', () => {
  const listeners = new Map();
  const timers = [];
  const media = { paused: true, ended: false };
  const window = { location: { href: '' } }; window.top = window;
  const document = { readyState: 'complete', querySelectorAll: () => [media], addEventListener: (name, fn) => listeners.set(name, fn) };
  class Element { value = 'private form contents'; matches() { return true; } }
  const source = readFileSync(new URL('../src-tauri/src/window/page_activity.js', import.meta.url), 'utf8').replace('__ACTIVITY_TOKEN__', '"test-generation"');
  vm.runInNewContext(source, { window, document, Element, URLSearchParams, setTimeout: fn => timers.push(fn) });
  const fire = name => { listeners.get(name)({ target: new Element() }); timers.splice(0).forEach(fn => fn()); return new URL(window.location.href); };
  assert.equal(fire('input').searchParams.get('dirty'), 'true');
  assert.ok(!window.location.href.includes('private'));
  assert.equal(fire('reset').searchParams.get('dirty'), 'false');
  media.paused = false;
  assert.equal(fire('play').searchParams.get('media'), 'true');
  media.paused = true;
  assert.equal(fire('pause').searchParams.get('media'), 'false');
});
