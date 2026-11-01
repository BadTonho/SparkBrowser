import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createNoteWriter } from '../src/lib/services/note-writer.js';
test('note writes retain newer drafts, serialize deletion and recover after errors', async () => {
  let release; let fail = false; const writes = [];
  const writer = createNoteWriter({ delay: 60_000, changed() {}, async save(url, title, text) { writes.push([url, text]); if (fail) throw Error('offline'); if (text === 'first') await new Promise(resolve => release = resolve); }, async remove(url) { writes.push([url, 'DELETED']); } });
  writer.edit('a', 'A', 'first'); const first = writer.flush('a');
  await new Promise(resolve => setImmediate(resolve));
  writer.edit('a', 'A', 'newest'); writer.edit('b', 'B', 'other page');
  release(); await first; assert.equal(writer.drafts.get('a').text, 'newest'); assert.notEqual(writer.drafts.get('a').status, 'Saved');
  await writer.flush('a'); await writer.flush('b'); assert.deepEqual(writes.slice(-2), [['a', 'newest'], ['b', 'other page']]);
  fail = true; writer.edit('a', 'A', 'retained'); await writer.flush('a'); assert.equal(writer.drafts.get('a').status, 'Error');
  fail = false; await writer.flush('a'); assert.equal(writer.drafts.get('a').status, 'Saved');
  await writer.remove('a'); assert.deepEqual(writes.at(-1), ['a', 'DELETED']); assert.equal(writer.drafts.get('a').text, '');
});
