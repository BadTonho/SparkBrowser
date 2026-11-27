import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

function cardFixture() {
  const elements = new Map(['tab-hover-card', 'tab-hover-title', 'tab-hover-memory', 'tab-hover-image', 'tab-hover-empty']
    .map(id => [id, {
      id, textContent: '', style: {}, hidden: true, src: '',
      removeAttribute(name) { if (name === 'src') this.src = ''; },
    }]));
  const document = { documentElement: { lang: '' }, getElementById: id => elements.get(id) };
  const window = {};
  runInNewContext(readFileSync('public/overlays/tab-hover-card.js', 'utf8'), { window, document });
  let revision = 0;
  const paint = data => window.setTabHoverCard({
    revision: ++revision, locale: 'en', title: 'Tab', memory: 'Memory usage: 80 MB', noPreview: 'No preview',
    picture: null, ...data,
  });
  return { elements, document, paint, window };
}

test('the card shows when painted and clears when hidden, ready for the next tab', () => {
  const { elements, paint, window } = cardFixture(), card = elements.get('tab-hover-card');
  paint({ title: 'GitHub' });
  assert.equal(card.hidden, false);
  assert.equal(elements.get('tab-hover-title').textContent, 'GitHub');
  window.hideTabHoverCard();
  assert.equal(card.hidden, true);
  paint({ title: 'Docs' });
  assert.equal(card.hidden, false);
  assert.equal(elements.get('tab-hover-title').textContent, 'Docs');
});

test('only pictures Spark took are shown, otherwise the card says there is no preview', () => {
  const { elements, paint } = cardFixture();
  const image = elements.get('tab-hover-image'), empty = elements.get('tab-hover-empty');
  paint({ picture: 'data:image/jpeg;base64,AAAA' });
  assert.equal(image.hidden, false);
  assert.equal(image.src, 'data:image/jpeg;base64,AAAA');
  assert.equal(empty.hidden, true);
  paint({ picture: 'https://example.test/tracker.png' });
  assert.equal(image.hidden, true);
  assert.equal(image.src, '');
  assert.equal(empty.textContent, 'No preview');
});

test('text is written as text and a late paint cannot replace a newer one', () => {
  const { elements, window, document } = cardFixture();
  const base = { locale: 'pt-BR', memory: 'Uso de memória: 10 MB', noPreview: '', picture: null };
  window.setTabHoverCard({ ...base, revision: 5, title: '<img src=x onerror=alert(1)>' });
  window.setTabHoverCard({ ...base, revision: 4, title: 'Old', locale: 'en' });
  assert.equal(elements.get('tab-hover-title').textContent, '<img src=x onerror=alert(1)>');
  assert.equal(document.documentElement.lang, 'pt-BR');
});

test('long tab titles are shortened to one line', async () => {
  // A Windows checkout has CRLF line endings; the pattern below expects LF.
  const source = readFileSync('src/lib/actions/tabHoverCard.ts', 'utf8').replace(/\r\n/g, '\n');
  const body = source.match(/export function shortTitle[\s\S]*?\n}\n/)[0]
    .replace('(title: string, max = MAX_TITLE): string', '(title, max = 60)');
  const shortTitle = new Function(`${body.replace('export ', '')}; return shortTitle;`)();
  assert.equal(shortTitle('  GitHub   -   Spark\n'), 'GitHub - Spark');
  const long = shortTitle('a'.repeat(100));
  assert.equal(long.length, 60);
  assert.ok(long.endsWith('…'));
});
