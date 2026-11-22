import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { SEARCH_ENGINES, DEFAULT_SEARCH_ENGINE, buildSearchUrl, findSearchEngine, isSearchEngineId } from '../src/lib/services/search-engines.js';
import { resolvePreviewInput } from '../src/lib/services/site-search.js';

const rust = readFileSync(new URL('../src-tauri/src/search_engines.rs', import.meta.url), 'utf8');
const field = (name) => [...rust.matchAll(new RegExp(`\\b${name}: "([^"]*)"`, 'g'))].map((match) => match[1]);
const catalog = (language) => JSON.parse(readFileSync(new URL(`../src/lib/i18n/catalogs/${language}/settings.json`, import.meta.url), 'utf8'));

test('the front-end list matches the native list, in order, with the same URLs and parameters', () => {
  assert.deepEqual(SEARCH_ENGINES.map((engine) => engine.id), field('id'));
  assert.deepEqual(SEARCH_ENGINES.map((engine) => engine.label), field('label'));
  assert.deepEqual(SEARCH_ENGINES.map((engine) => engine.home), field('home'));
  assert.deepEqual(SEARCH_ENGINES.map((engine) => engine.search), field('search'));
  assert.deepEqual(SEARCH_ENGINES.map((engine) => engine.param), field('param'));
  assert.equal(rust.match(/DEFAULT_ID: &str = "([^"]+)"/)[1], DEFAULT_SEARCH_ENGINE);
  assert.equal(SEARCH_ENGINES[0].id, DEFAULT_SEARCH_ENGINE, 'unknown ids fall back to the first entry');
});

test('every engine has a unique id, https URLs, a color and a description in both languages', () => {
  assert.equal(new Set(SEARCH_ENGINES.map((engine) => engine.id)).size, SEARCH_ENGINES.length);
  for (const engine of SEARCH_ENGINES) {
    assert.match(engine.color, /^#[0-9A-F]{6}$/i);
    for (const url of [engine.home, engine.search]) assert.equal(new URL(url).protocol, 'https:', engine.id);
    for (const language of ['en', 'pt-BR']) {
      assert.ok(catalog(language)[`settings.engine_${engine.id}_desc`], `${language} description for ${engine.id}`);
    }
  }
});

test('searches use each engine\'s own parameter and encode the query', () => {
  for (const engine of SEARCH_ENGINES) {
    const url = new URL(buildSearchUrl(engine.id, 'rust & svelte'));
    assert.equal(url.host, new URL(engine.search).host);
    assert.equal([...url.searchParams.keys()].join(), engine.param);
    assert.equal(url.searchParams.get(engine.param), 'rust & svelte');
  }
  assert.ok(new URL(buildSearchUrl('yahoo', 'a b')).searchParams.has('p'));
});

test('unknown engines fall back to the default and the omnibox preview follows the chosen engine', () => {
  assert.equal(findSearchEngine('gone').id, DEFAULT_SEARCH_ENGINE);
  assert.equal(isSearchEngineId('bing'), true);
  assert.equal(isSearchEngineId('Bing'), false);
  assert.equal(new URL(resolvePreviewInput('missing term', [], 'brave')).hostname, 'search.brave.com');
  assert.equal(new URL(resolvePreviewInput('missing term', [], 'nonsense')).hostname, 'www.google.com');
});

test('every engine has one bundled logo', async () => {
  const { SEARCH_ENGINE_ICON_PATHS } = await import('../src/lib/components/icons/searchEngineIcons.js');
  const bundledLogos = ['google', 'duckduckgo'];
  assert.deepEqual([...Object.keys(SEARCH_ENGINE_ICON_PATHS), ...bundledLogos].sort(), SEARCH_ENGINES.map((engine) => engine.id).sort());
  for (const [id, path] of Object.entries(SEARCH_ENGINE_ICON_PATHS)) {
    // Path data only: no markup, scripts or external references can ride along.
    assert.match(path, /^[Mm][MmLlHhVvCcSsQqTtAaZz0-9eE+\-., ]*$/, id);
    assert.ok(path.length > 40 && path.length < 4000, id);
  }
  const component = readFileSync(new URL('../src/lib/components/icons/SearchEngineIcon.svelte', import.meta.url), 'utf8');
  for (const id of bundledLogos) {
    const file = `./search-engines/${id}.svg`;
    const svg = readFileSync(new URL(`../src/lib/components/icons/${file.slice(2)}`, import.meta.url), 'utf8');
    assert.ok(component.includes(`url("${file}")`), id);
    assert.match(svg, /<path\s/, id);
    assert.doesNotMatch(svg, /<(?:script|foreignObject|image)\b|\b(?:href|on\w+)\s*=|url\(/i, id);
  }
});

test('Google is the default engine for new profiles', () => {
  assert.equal(DEFAULT_SEARCH_ENGINE, 'google');
  assert.equal(SEARCH_ENGINES[0].id, 'google');
});
