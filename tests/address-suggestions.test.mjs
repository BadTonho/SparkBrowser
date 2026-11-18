import assert from 'node:assert/strict';
import { test } from 'node:test';
import { buildSuggestions, looksLikeAddress, normalizeAddress, groupHistory } from '../src/lib/services/address-suggestions.js';

const NOW = 1_800_000_000;
const row = (id, title, url, daysAgo = 0) => ({ id, title, url, visited_at: NOW - daysAgo * 86400 });

test('addresses are told apart from words to search for', () => {
  for (const text of ['example.com', 'https://example.com/a?b=1', 'localhost:1420', '192.168.0.1:8080/x', 'sub.site.co.uk/page']) {
    assert.equal(looksLikeAddress(text), true, text);
  }
  for (const text of ['', 'hello world', 'rust lifetimes', 'what is example.com', 'cats']) {
    assert.equal(looksLikeAddress(text), false, text);
  }
});

test('the same page is not listed twice, whatever its scheme, www or trailing slash', () => {
  assert.equal(normalizeAddress('https://www.Example.com/'), 'example.com');
  assert.equal(normalizeAddress('http://example.com'), 'example.com');
  const grouped = groupHistory([row(1, 'A', 'https://example.com/', 1), row(2, 'A', 'http://www.example.com', 0), row(3, 'B', 'https://other.org')]);
  assert.equal(grouped.length, 2);
  assert.equal(grouped.find(item => normalizeAddress(item.row.url) === 'example.com').visits, 2);
});

test('an empty address bar offers the most recent sites, one per address', () => {
  const list = buildSuggestions({ query: '', history: [row(1, 'News', 'https://news.test/'), row(2, 'News again', 'https://news.test'), row(3, 'Docs', 'https://docs.test')], bookmarks: [], now: NOW });
  assert.deepEqual(list.map(item => item.kind), ['history', 'history']);
  assert.deepEqual(list.map(item => item.target), ['https://news.test/', 'https://docs.test']);
});

test('typed words come first as a search, then matching bookmarks, then visited pages', () => {
  const list = buildSuggestions({
    query: 'rust',
    history: [row(1, 'Rust book', 'https://doc.rust-lang.org/book/', 2), row(2, 'Cooking', 'https://food.test')],
    bookmarks: [{ title: 'Rust playground', url: 'https://play.rust-lang.org/' }],
    engineLabel: 'Google',
    now: NOW,
  });
  assert.deepEqual(list.map(item => item.kind), ['search', 'bookmark', 'history']);
  assert.equal(list[0].target, 'rust');
  assert.equal(list[0].secondary, 'Google');
});

test('typed text that is an address leads with going there, and its own history row is not repeated', () => {
  const list = buildSuggestions({ query: 'example.com', history: [row(1, 'Example', 'https://example.com/', 0), row(2, 'Example docs', 'https://example.com/docs', 0)], bookmarks: [], now: NOW });
  assert.equal(list[0].kind, 'navigate');
  assert.deepEqual(list.map(item => item.target), ['example.com', 'https://example.com/docs']);
});

test('pages that start with the query and are visited often rank above a passing mention', () => {
  const list = buildSuggestions({
    query: 'git',
    history: [
      row(1, 'Why I left git', 'https://blog.test/post', 0),
      row(2, 'GitHub', 'https://github.com/', 3), row(3, 'GitHub', 'https://github.com/', 4), row(4, 'GitHub', 'https://github.com/', 5),
    ],
    bookmarks: [],
    now: NOW,
  });
  assert.equal(list[1].target, 'https://github.com/');
});

test('every typed word has to match, in the title or the address', () => {
  const list = buildSuggestions({ query: 'rust book', history: [row(1, 'Rust book', 'https://a.test'), row(2, 'Rust news', 'https://b.test')], bookmarks: [], now: NOW });
  assert.deepEqual(list.slice(1).map(item => item.target), ['https://a.test']);
});

test('search terms from the engine are added last, without repeating the typed text', () => {
  const list = buildSuggestions({ query: 'rust', history: [], bookmarks: [], webTerms: ['rust', 'rust lang', 'rust book', '  '], now: NOW });
  assert.deepEqual(list.map(item => `${item.kind}:${item.target}`), ['search:rust', 'web:rust lang', 'web:rust book']);
});

test('the list never grows past its limit', () => {
  const history = Array.from({ length: 30 }, (_, index) => row(index + 1, `Page ${index}`, `https://site${index}.test`));
  assert.equal(buildSuggestions({ query: 'page', history, bookmarks: [], now: NOW }).length, 8);
  assert.equal(buildSuggestions({ query: '', history, bookmarks: [], now: NOW }).length, 8);
  assert.equal(buildSuggestions({ query: 'page', history, bookmarks: [], now: NOW, limit: 3 }).length, 3);
});
