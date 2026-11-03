import test from 'node:test';
import assert from 'node:assert/strict';
import { focusedTabs, replaceGroupOrder, splitHeaderGeometry } from '../src/lib/services/split-tab-groups.js';

const tabs = ['a','b','c','d'].map(id => ({ id, workspace_id: 'ws' }));
const split = { workspace_id:'ws', left_tab_id:'a', right_tab_id:'b', focused_tab_id:'b', left_tab_ids:['a','c'], right_tab_ids:['b','d'] };
test('focused shortcuts use only the owning group, and normal mode uses the workspace', () => {
  assert.deepEqual(focusedTabs(tabs, split, 'ws').map(t => t.id), ['b','d']);
  assert.deepEqual(focusedTabs(tabs, {...split, focused_tab_id:'a'}, 'ws').map(t => t.id), ['a','c']);
  assert.deepEqual(focusedTabs(tabs, null, 'ws'), tabs);
});
test('group reordering preserves every position belonging to another group or workspace', () => {
  assert.deepEqual(replaceGroupOrder([...tabs, {id:'other',workspace_id:'work'}], ['c','a']), ['c','b','a','d','other']);
});
test('header geometry follows logical native pane bounds including the gap and minimum width', () => {
  for (const width of [662, 980, 1262]) for (const ratio of [.2,.5,.8]) {
    const g = splitHeaderGeometry({ x:9, width }, ratio);
    assert.equal(g.right, g.left + g.leftWidth + 8);
    assert.equal(g.leftWidth + g.rightWidth + 8, width);
    assert.ok(g.leftWidth >= 100 && g.rightWidth >= 100);
  }
});
