import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

function click(href, options = {}) {
  const anchor = { href, hasAttribute: name => name === 'download' && Boolean(options.download) };
  class Element { closest() { return anchor; } }
  const window = {};
  let handler;
  const document = { baseURI: 'https://example.test/', addEventListener: (_, callback) => handler = callback };
  const settings = { enabled: true, exceptions: [], ...options.settings };
  const source = readFileSync(new URL('../src-tauri/src/privacy/tracking_clicks.js', import.meta.url), 'utf8')
    .replace('__SETTINGS__', JSON.stringify(settings)).replace('__TOKEN__', JSON.stringify('fixture'));
  vm.runInNewContext(source, { window, document, Element, URL, URLSearchParams });
  handler({ isTrusted: true, defaultPrevented: false, button: 0, target: new Element() });
  return anchor.href;
}

test('cleans clicked GET links, but preserves authentication, signed links and exceptions', () => {
  assert.equal(click('https://example.test/p?utm_source=x&keep=%2F'), 'https://example.test/p?keep=%2F');
  for (const input of ['https://example.test/login?utm_source=x', 'https://example.test/?state=x&utm_source=x', 'https://example.test/?X-Amz-Signature=x&utm_source=x']) assert.equal(click(input), input);
  const url = 'https://example.test/?utm_source=x';
  assert.equal(click(url, { settings: { enabled: false } }), url);
  assert.equal(click(url, { settings: { exceptions: ['example.test'] } }), url);
  assert.equal(click(url, { download: true }), url);
});
