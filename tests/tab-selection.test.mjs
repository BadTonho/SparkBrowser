import assert from 'node:assert/strict';
import { test } from 'node:test';
import { resolveTabSelection } from '../src/lib/services/tab-selection.js';

const ids = ['a', 'b', 'c', 'd'];

test('Ctrl toggles individual tabs and preserves list order', () => {
  const first = resolveTabSelection([], null, ids, 'c', { ctrlKey: true });
  assert.deepEqual(first, { selectedIds: ['c'], anchorId: 'c' });
  assert.deepEqual(resolveTabSelection(first.selectedIds, first.anchorId, ids, 'a', { ctrlKey: true }), {
    selectedIds: ['a', 'c'], anchorId: 'a',
  });
  assert.deepEqual(resolveTabSelection(['a', 'c'], 'a', ids, 'c', { ctrlKey: true }), {
    selectedIds: ['a'], anchorId: 'c',
  });
});

test('Shift selects the inclusive range from the anchor; Ctrl+Shift adds that range', () => {
  assert.deepEqual(resolveTabSelection(['b'], 'b', ids, 'd', { shiftKey: true }), {
    selectedIds: ['b', 'c', 'd'], anchorId: 'b',
  });
  assert.deepEqual(resolveTabSelection(['a'], 'b', ids, 'd', { ctrlKey: true, shiftKey: true }), {
    selectedIds: ['a', 'b', 'c', 'd'], anchorId: 'b',
  });
});

test('plain activation clears selection and out-of-scope ids cannot be selected', () => {
  assert.deepEqual(resolveTabSelection(['a'], 'a', ids, 'b', {}), { selectedIds: [], anchorId: null });
  assert.deepEqual(resolveTabSelection(['a'], 'a', ids, 'other-workspace-tab', { ctrlKey: true }), {
    selectedIds: [], anchorId: null,
  });
});
