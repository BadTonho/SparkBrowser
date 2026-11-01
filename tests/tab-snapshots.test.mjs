import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createSnapshotGate } from '../src/lib/services/tab-snapshots.js';

test('late command replies cannot resurrect a tab closed by a newer event', () => {
  const accept = createSnapshotGate();
  assert.deepEqual(accept({ revision: 2, tabs: [{ id: 'remaining' }] }), [{ id: 'remaining' }]);
  assert.equal(accept({ revision: 1, tabs: [{ id: 'closed' }, { id: 'remaining' }] }), undefined);
  assert.equal(accept({ revision: 2, tabs: [] }), undefined);
  assert.deepEqual(accept({ revision: 3, tabs: [] }), []);
});
