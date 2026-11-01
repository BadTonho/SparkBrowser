import assert from 'node:assert/strict';
import { test } from 'node:test';
import { validateSiteSearch, resolvePreviewInput } from '../src/lib/services/site-search.js';
test('site search encodes terms once and only accepts path/query placeholders', () => {
  const item = { keyword: 'gh', name: 'GitHub', template: 'https://github.com/search?q={query}' };
  validateSiteSearch(item);
  const resolved = new URL(resolvePreviewInput('GH ação & + /%', [item]));
  assert.equal(resolved.searchParams.get('q'), 'ação & + /%');
  assert.equal(new URL(resolvePreviewInput('missing term', [item], 'duckduckgo')).hostname, 'duckduckgo.com');
  for (const template of ['https://{query}.test/', 'https://example.test/#{query}', 'javascript:{query}', 'https://example.test/?q={query}&x={query}']) assert.throws(() => validateSiteSearch({ ...item, template }));
});
