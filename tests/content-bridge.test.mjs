import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import { test } from 'node:test';
import assert from 'node:assert/strict';

const BINDING = '__sparkCSfixturefixturefixture';

function bridge(engine, { binding, handler } = {}) {
  const timers = [], messages = [];
  const window = { location: { href: 'https://fixture.test/original' } }; window.top = window;
  if (binding) window[BINDING] = message => messages.push(JSON.parse(message));
  if (handler) window.webkit = { messageHandlers: { sparkContent: { postMessage(message) { messages.push(JSON.parse(message)); } } } };
  const context = vm.createContext({ window, setTimeout: callback => { timers.push(callback); return timers.length; } });
  vm.runInContext(readFileSync('src-tauri/src/window/content_bridge.js', 'utf8')
    .replace('__CONTENT_NATIVE__', JSON.stringify(engine))
    .replace('__CONTENT_BINDING__', JSON.stringify(BINDING)), context);
  return { window, messages, context, flush: () => timers.splice(0).forEach(callback => callback()) };
}

test('the native binding is taken before the page runs and is no longer reachable from the page', () => {
  const f = bridge('windows', { binding: true });
  assert.equal(Object.prototype.hasOwnProperty.call(f.window, BINDING), false);
  f.window.__sparkPostContent('spark-internal://autofill/ready?token=secret');
  assert.deepEqual(f.messages.map(message => message.sparkContent), ['spark-internal://autofill/ready?token=secret']);
});

test('a page that replaces globals and prototypes afterwards sees nothing of the messages', () => {
  const f = bridge('windows', { binding: true });
  const seen = [];
  vm.runInContext(`
    const record = value => { window.seen.push(String(value)); };
    JSON.stringify = value => { record(value); return '{}'; };
    Array.prototype.push = function (...values) { values.forEach(record); return 0; };
    Object.defineProperty(Object.prototype, 'next', { set: record, get() { return undefined; }, configurable: true });
    Object.defineProperty(Object.prototype, 'message', { set: record, get() { return undefined; }, configurable: true });
  `, f.context);
  f.window.seen = seen;
  f.window.__sparkPostContent('spark-internal://autofill/ready?token=secret');
  assert.deepEqual(seen, []);
  assert.equal(f.messages[0].sparkContent, 'spark-internal://autofill/ready?token=secret');
});

test('messages wait for a late binding under the random name and never navigate', () => {
  const f = bridge('windows');
  f.window.__sparkPostContent('spark-internal://new-tab/read?sequence=1');
  f.window.__sparkPostContent('spark-internal://page-activity/state?sequence=2');
  // A function planted under the old, well-known name is ignored.
  const stolen = [];
  f.window.__sparkContentSignal = message => stolen.push(message);
  f.flush();
  assert.equal(f.window.location.href, 'https://fixture.test/original');
  f.window[BINDING] = message => f.messages.push(JSON.parse(message));
  f.flush();
  assert.deepEqual(f.messages.map(message => message.sparkContent), [
    'spark-internal://new-tab/read?sequence=1', 'spark-internal://page-activity/state?sequence=2',
  ]);
  assert.deepEqual(stolen, []);
  assert.equal(f.window.location.href, 'https://fixture.test/original');
});

test('WebKitGTK sends through the script message handler instead of navigating', () => {
  const f = bridge('linux', { handler: true });
  f.window.webkit = { messageHandlers: { sparkContent: { postMessage() { throw new Error('replaced by the page'); } } } };
  f.window.__sparkPostContent('spark-internal://autofill/ready?token=secret');
  assert.equal(f.messages[0].sparkContent, 'spark-internal://autofill/ready?token=secret');
  assert.equal(f.window.location.href, 'https://fixture.test/original');
});

test('engines without a native channel keep the authenticated navigation transport', () => {
  const f = bridge('none');
  f.window.__sparkPostContent('spark-internal://new-tab/read?token=fixture');
  assert.equal(f.window.location.href, 'spark-internal://new-tab/read?token=fixture');
});
