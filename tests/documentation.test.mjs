import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import assert from 'node:assert/strict';
import { test } from 'node:test';
const list = (root, prefix = '') => readdirSync(join(root, prefix), { withFileTypes: true }).flatMap(item => item.isDirectory() ? list(root, join(prefix, item.name)) : [join(prefix, item.name)]).sort();

test('documentation translations mirror the English tree and have valid text encoding', () => {
  const english = list('docs/en');
  assert.deepEqual(list('docs/pt-BR'), english);
  for (const language of ['en', 'pt-BR']) {
    for (const file of english.filter(file => file.endsWith('.md'))) {
      const text = readFileSync(join('docs', language, file), 'utf8');
      assert.ok(!text.includes('\uFFFD'), `${language}/${file} has invalid UTF-8`);
      assert.ok(!/[\p{L}]\?\?[\p{L}]/u.test(text), `${language}/${file} has corrupted accents`);
    }
  }
});
