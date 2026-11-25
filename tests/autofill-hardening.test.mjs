import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import assert from 'node:assert/strict';

// The autofill script shares its JavaScript world with the page. Whatever touches the channel token,
// the item ids or the functions that act for the person must use functions captured at start, never
// globals or prototype methods looked up later (the page can replace those).
const source = readFileSync('src-tauri/src/autofill/autofill.js', 'utf8')
  .split('\n').filter(line => !/^\s*(\/\/|\/\*\*|\*)/.test(line)).join('\n');

test('the token never goes through page-replaceable APIs or a navigation', () => {
  assert.doesNotMatch(source, /URLSearchParams/);
  assert.doesNotMatch(source, /location\.href\s*=/);
  assert.doesNotMatch(source, /window\.__sparkPostContent\(/, 'the transport is captured at start');
  assert.match(source, /const send = window\.__sparkPostContent;/);
});

test('handlers are registered and events are read through captured functions', () => {
  assert.doesNotMatch(source, /\.addEventListener\(/, 'a replaced addEventListener would receive the handlers');
  assert.doesNotMatch(source, /event\.(target|key|preventDefault)\b/);
  assert.match(source, /const listen = EventTarget\.prototype\.addEventListener;/);
});

test('the suggestion list keeps its choices out of Array.prototype', () => {
  const show = source.slice(source.indexOf('function show('), source.indexOf('function highlight('));
  assert.doesNotMatch(show, /\.push\(|for \(const .* of /);
  assert.doesNotMatch(show, /document\.createElement|\.attachShadow\(|\.textContent =/);
});
