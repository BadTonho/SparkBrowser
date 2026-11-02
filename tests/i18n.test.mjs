import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync, readdirSync } from 'node:fs';
import { resolveLanguage, interpolate } from '../src/lib/i18n/language.js';
import { describeError, errorDetails, UserError } from '../src/lib/services/user-errors.js';

test('English and Portuguese catalogs have identical keys and interpolation parameters', () => {
  const root = 'src/lib/i18n/catalogs';
  const files = readdirSync(`${root}/en`).sort();
  assert.deepEqual(readdirSync(`${root}/pt-BR`).sort(), files);
  for (const file of files) {
    const en = JSON.parse(readFileSync(`${root}/en/${file}`, 'utf8'));
    const pt = JSON.parse(readFileSync(`${root}/pt-BR/${file}`, 'utf8'));
    assert.deepEqual(Object.keys(pt).sort(), Object.keys(en).sort(), file);
    for (const key of Object.keys(en)) {
      const parameters = value => [...value.matchAll(/\{(\w+)\}/g)].map(match => match[1]).sort();
      assert.deepEqual(parameters(pt[key]), parameters(en[key]), key);
      assert.ok(pt[key].trim() && !pt[key].includes('\uFFFD'), key);
    }
  }
});
test('system language fallback, explicit preference and literal user interpolation', () => {
  assert.equal(resolveLanguage('system', ['pt-PT']), 'pt-BR');
  assert.equal(resolveLanguage('system', ['es', 'pt-BR']), 'en');
  assert.equal(resolveLanguage('system', []), 'en');
  assert.equal(resolveLanguage('en', ['pt-BR']), 'en');
  assert.equal(resolveLanguage('pt-BR', ['en']), 'pt-BR');
  assert.equal(interpolate('Open {name}', { name: '<b>{query}</b>' }), 'Open <b>{query}</b>');
});
test('user error codes remain stable independently from technical error language', () => {
  for (const detail of ['English details', 'Detalhes técnicos', '<script>untrusted</script>']) {
    const value = describeError(new UserError('notes.flush_failed', detail));
    assert.deepEqual(errorDetails(value), { code: 'notes.flush_failed', detail });
  }
  assert.equal(errorDetails('Notes require an HTTP/HTTPS page').code, 'operation.failed');
});
