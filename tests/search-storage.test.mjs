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

test('command ranking reserves navigation and caps saved categories at two each', () => {
  const item = (id,title) => ({id,title,subtitle:'',action(){}});
  const all=['switch-','ws-','bm-','cmd-','note-','session-'].flatMap(prefix=>Array.from({length:8},(_,i)=>item(prefix+i,'Café '+i)));
  all.push(item('open-new-tab','go'),item('navigate-current','go'));
  const ranked=rankCommands(all,'cafe');
  assert.equal(ranked.length,12);
  assert.ok(ranked.some(i=>i.id==='open-new-tab'));assert.ok(ranked.some(i=>i.id==='navigate-current'));
  for(const prefix of ['ws-','bm-','cmd-','note-','session-'])assert.ok(ranked.filter(i=>i.id.startsWith(prefix)).length<=2);
  assert.ok(ranked.filter(i=>i.id.startsWith('switch-')).length<=3);
  assert.equal(rankCommands([item('note-content','unrelated title')],'content deep inside note').length,1);
});
