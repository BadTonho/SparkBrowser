import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import assert from 'node:assert/strict';

// The keyboard script shares its JavaScript world with the page: the token and every handler that
// sends a message must only meet functions captured at start (see docs/en/security).
const source = readFileSync('src-tauri/src/window/keyboard_bridge.js', 'utf8')
  .split('\n').filter(line => !/^\s*(\/\/|\/\*\*|\*)/.test(line)).join('\n');

test('messages use the captured transport and never a navigation', () => {
  assert.doesNotMatch(source, /URLSearchParams|location\.href\s*=/);
  assert.match(source, /const send = window\.__sparkPostContent;/);
  assert.doesNotMatch(source, /window\.__sparkPostContent\(/);
});

test('handlers, timers and event properties come from captured functions', () => {
  assert.doesNotMatch(source, /\.addEventListener\(|[^.\w]setTimeout\(/);
  assert.doesNotMatch(source, /event\.(key|target|button|ctrlKey|altKey|shiftKey|metaKey|repeat|defaultPrevented|preventDefault|stopImmediatePropagation)\b/);
  assert.doesNotMatch(source, /\.push\(|\.shift\(\)|for \(const .* of /);
});

test('the Rust side fills every placeholder', () => {
  const rust = readFileSync('src-tauri/src/window/keyboard_bridge.rs', 'utf8');
  for (const name of source.match(/__[A-Z_]+__/g)) assert.ok(rust.includes(`"${name}"`), name);
});
