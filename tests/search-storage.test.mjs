import { test } from 'node:test';
import assert from 'node:assert/strict';
import { rankCommands } from '../src/lib/services/command-ranking.js';
import { createStorageAdapter } from '../src/lib/services/storage.js';

test('command matches outrank incidental URLs and tabs cannot exclude other categories', () => {
  const item = (id, title) => ({ id, title, subtitle: '', action() {} });
  const tabs = Array.from({ length: 50 }, (_, i) => item(`switch-${i}`, 'settings page'));
  const ranked = rankCommands([...tabs, item('cmd-settings', 'Settings'), item('ws-1', 'Settings workspace')], 'settings');
  assert.equal(ranked[0].id, 'cmd-settings');
  assert.equal(ranked.filter(item => item.id.startsWith('switch-')).length, 3);
  assert.ok(ranked.some(item => item.id === 'ws-1'));
  assert.equal(rankCommands([item('cmd-1', 'Ação')], 'acao').length, 1);
});

test('denied storage getters and quota errors keep in-memory preferences usable', () => {
  const errors = [];
  const denied = createStorageAdapter(() => { throw new Error('SecurityError'); }, op => errors.push(op));
  assert.equal(denied.getItem('preference'), null);
  assert.equal(denied.setItem('preference', 'value'), false);
  assert.equal(denied.removeItem('preference'), false);
  assert.deepEqual(errors, ['read', 'write', 'write']);
  const quota = createStorageAdapter(() => ({ getItem: () => 'saved', setItem() { throw new Error('QuotaExceededError'); } }), () => {});
  assert.equal(quota.getItem('preference'), 'saved');
  assert.equal(quota.setItem('preference', 'next'), false);
});
