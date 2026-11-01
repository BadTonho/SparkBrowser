import assert from 'node:assert/strict';
import { test } from 'node:test';
import { duplicateGroups, duplicateCandidates } from '../src/lib/services/tab-organizer.js';
test('duplicate review preserves active/recent/protected tabs and distinguishes fragments and queries', () => {
  const make = (id, url, props = {}) => ({ id, url, title: id, workspace_id: 'ws', is_active: false, is_hibernated: false, ...props });
  const tabs = [make('active','https://EXAMPLE.test/a',{is_active:true}),make('extra','https://example.test/a'),make('pinned','https://example.test/a',{is_pinned:true}),make('fragment','https://example.test/a#x'),make('query','https://example.test/a?q=1'),make('older','https://other.test/',{last_active_epoch:1}),make('recent','https://other.test/',{last_active_epoch:2})];
  assert.equal(duplicateGroups(tabs).length,2); assert.deepEqual(duplicateCandidates(tabs),['extra','older']);
});
