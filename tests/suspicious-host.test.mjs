import test from 'node:test';
import assert from 'node:assert/strict';
import { suspiciousHost } from '../src/lib/services/suspicious-host.js';
test('warns on close local targets and punycode without blocking exact sites', () => {
  assert.equal(suspiciousHost('https://gooogle.com/', [])?.target, 'google.com');
  assert.equal(suspiciousHost('https://mybokmarks.test/', ['https://mybookmarks.test/'])?.target, 'mybookmarks.test');
  assert.equal(suspiciousHost('https://xn--80ak6aa92e.com/', [])?.host, 'xn--80ak6aa92e.com');
  assert.equal(suspiciousHost('https://accounts.google.com/', []), null);
  assert.equal(suspiciousHost('https://gooogle.com/', ['https://gooogle.com/']), null);
  assert.equal(suspiciousHost('veloce://newtab', []), null);
  assert.equal(suspiciousHost('https://ordinary.example/', []), null);
  assert.equal(suspiciousHost('https://mybookmarks.test/', ['https://mybookmarks.test/']), null);
  assert.equal(suspiciousHost('https://notmybookmarks.test/', ['https://mybookmarks.test/']), null);
  assert.equal(suspiciousHost('https://gooogle.example/', []), null);
});
