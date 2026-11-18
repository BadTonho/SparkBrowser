import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { splitDropSide, splitEdgeWidth, crossedDragThreshold } from '../src/lib/services/tab-drag-geometry.js';

test('drag threshold uses either axis and starts at six pixels', () => {
  assert.equal(crossedDragThreshold(10, 10, 15, 10), false);
  assert.equal(crossedDragThreshold(10, 10, 10, 16), true);
  assert.equal(crossedDragThreshold(10, 10, 16, 10), true);
  assert.equal(crossedDragThreshold(10, 10, 15, 15), true);
});
test('docking is offered in the outer 15% of the page edges, between 96 and 240 pixels', () => {
  const rect = { left: 280, top: 80, width: 1000, height: 700 };
  // 15% of 1000 is 150 pixels on each side.
  for (const x of [280, 300, 430]) assert.equal(splitDropSide(rect, x, 300), 'left');
  for (const x of [1130, 1270, 1280]) assert.equal(splitDropSide(rect, x, 300), 'right');
  for (const x of [279, 431, 700, 1129, 1281]) assert.equal(splitDropSide(rect, x, 300), null);
  assert.equal(splitEdgeWidth(400), 96, 'a narrow page keeps a usable zone');
  assert.equal(splitEdgeWidth(3000), 240, 'a very wide page does not give up half of itself');
  assert.equal(splitDropSide({ left: 0, top: 0, width: 208, height: 300 }, 96, 100), 'left');
  assert.equal(splitDropSide({ left: 0, top: 0, width: 208, height: 300 }, 97, 100), null);
  assert.equal(splitDropSide(rect, 300, 79), null);
  assert.equal(splitDropSide(rect, 300, 781), null);
  assert.equal(splitDropSide({ ...rect, width: 150 }, 300, 300), null);
});

function previewFixture() {
  const elements = new Map(['split-drop-label', 'split-drop-preview', 'split-companion-placeholder', 'dragged-tab', 'dragged-tab-title', 'dragged-tab-host']
    .map(id => [id, { textContent: '', style: {}, hidden: false, getBoundingClientRect: () => ({ width: 244, height: 56 }) }]));
  const document = { documentElement: { lang: '' }, getElementById: id => elements.get(id) };
  const window = {};
  runInNewContext(readFileSync('public/overlays/tab-drag-preview.js', 'utf8'), { window, document, innerWidth: 800, innerHeight: 600 });
  const paint = data => window.setDragPreview({ revision: 1, label: '', locale: 'en', title: 'Tab title', host: 'example.test', pointer: { x: 300, y: 200 }, pane: null, ...data });
  return { elements, document, paint };
}

test('a delayed preview paint cannot overwrite a newer destination or locale', () => {
  const { elements, document, paint } = previewFixture(), label = elements.get('split-drop-label');
  paint({ revision: 12, label: 'Drop on right' });
  paint({ revision: 11, label: 'Solte à esquerda', locale: 'pt-BR' });
  assert.equal(label.textContent, 'Drop on right');
  assert.equal(document.documentElement.lang, 'en');
  paint({ revision: 13, label: 'Solte à esquerda', locale: 'pt-BR' });
  assert.equal(label.textContent, 'Solte à esquerda');
  assert.equal(document.documentElement.lang, 'pt-BR');
});

test('dragged tab follows the pointer outside docking regions and keeps its title as text', () => {
  const { elements, paint } = previewFixture();
  paint({ title: '<img src=x onerror=alert(1)>' });
  assert.equal(elements.get('split-drop-preview').hidden, true);
  assert.equal(elements.get('dragged-tab-title').textContent, '<img src=x onerror=alert(1)>');
  assert.equal(elements.get('dragged-tab-host').textContent, 'example.test');
  assert.deepEqual(elements.get('dragged-tab').style, { left: '316px', top: '218px' });
  paint({ revision: 2, pointer: { x: 795, y: 595 }, pane: { x: 280, y: 80, width: 250, height: 500 } });
  assert.deepEqual(elements.get('dragged-tab').style, { left: '535px', top: '521px' });
  assert.equal(elements.get('split-drop-preview').hidden, false);
  assert.deepEqual(elements.get('split-drop-preview').style, { left: '284px', top: '84px', width: '242px', height: '492px' });
  paint({ revision: 3, pointer: { x: -5, y: -5 } });
  assert.deepEqual(elements.get('dragged-tab').style, { left: '11px', top: '13px' });
  assert.equal(elements.get('split-drop-preview').hidden, true);
});

test('an unloaded companion reserves its final pane and old paints cannot restore it', () => {
  const { elements, paint } = previewFixture();
  const reserved = elements.get('split-companion-placeholder');
  const placeholder = { x: 10, y: 80, width: 392, height: 500 };
  paint({ revision: 10, placeholder });
  assert.equal(reserved.hidden, false);
  assert.deepEqual(reserved.style, { left: '10px', top: '80px', width: '392px', height: '500px' });
  paint({ revision: 11 });
  assert.equal(reserved.hidden, true);
  paint({ revision: 10, placeholder });
  assert.equal(reserved.hidden, true);
});
