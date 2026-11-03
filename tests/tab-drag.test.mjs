import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { splitDropSide, crossedDragThreshold } from '../src/lib/services/tab-drag-geometry.js';

test('drag threshold uses either axis and starts at six pixels', () => {
  assert.equal(crossedDragThreshold(10, 10, 15, 10), false);
  assert.equal(crossedDragThreshold(10, 10, 10, 16), true);
  assert.equal(crossedDragThreshold(10, 10, 16, 10), true);
  assert.equal(crossedDragThreshold(10, 10, 15, 15), true);
});
test('only the internal 32px of the page edges offer docking', () => {
  const rect = { left: 280, top: 80, width: 1000, height: 700 };
  for (const x of [280, 300, 312]) assert.equal(splitDropSide(rect, x, 300), 'left');
  for (const x of [1248, 1270, 1280]) assert.equal(splitDropSide(rect, x, 300), 'right');
  for (const x of [279, 313, 700, 1247, 1281]) assert.equal(splitDropSide(rect, x, 300), null);
  assert.equal(splitDropSide(rect, 300, 79), null);
  assert.equal(splitDropSide(rect, 300, 781), null);
  assert.equal(splitDropSide({ ...rect, width: 150 }, 300, 300), null);
});

test('a delayed preview paint cannot overwrite a newer destination or locale', () => {
  const label = { textContent: '' }, document = { documentElement: { lang: '' }, getElementById: () => label };
  const window = {};
  runInNewContext(readFileSync('public/overlays/tab-drag-preview.js', 'utf8'), { window, document });
  window.setDragPreview({ revision: 12, label: 'Drop on right', locale: 'en' });
  window.setDragPreview({ revision: 11, label: 'Solte à esquerda', locale: 'pt-BR' });
  assert.equal(label.textContent, 'Drop on right');
  assert.equal(document.documentElement.lang, 'en');
  window.setDragPreview({ revision: 13, label: 'Solte à esquerda', locale: 'pt-BR' });
  assert.equal(label.textContent, 'Solte à esquerda');
  assert.equal(document.documentElement.lang, 'pt-BR');
});
