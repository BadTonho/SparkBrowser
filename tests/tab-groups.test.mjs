import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

function writable(initial) {
  let value = initial; const listeners = new Set();
  const set = next => { value = next; listeners.forEach(listener => listener(value)); };
  return { set, update: edit => set(edit(value)), subscribe: listener => { listeners.add(listener); listener(value); return () => listeners.delete(listener); } };
}
// Values built inside the vm context are compared as plain JSON.
const get = store => { let value; store.subscribe(next => { value = next; })(); return JSON.parse(JSON.stringify(value ?? null)); };

function load({ saved = { groups: [], members: {} }, saves = [] } = {}) {
  const source = ts.transpileModule(readFileSync('src/lib/stores/tabGroupsStore.ts', 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const imports = {
    'svelte/store': { get, writable },
    '../services/ignored-error': { logIgnored() {} },
    '../services/bridge/runtime': { isTauriRuntime: () => true },
    '../services/bridge/tab-groups': { fetchTabGroups: async () => saved, saveTabGroups: async value => { saves.push(structuredClone(value)); } },
  };
  const exports = {};
  let uuid = 0;
  runInNewContext(source, { exports, require: name => imports[name], crypto: { randomUUID: () => `g${++uuid}` }, setTimeout, clearTimeout });
  return exports;
}

test('creating a group puts the tabs in it, and a tab can only be in one group', () => {
  const s = load();
  const first = s.createGroup(['a', 'b']);
  assert.deepEqual(get(s.tabGroups).members, { a: first, b: first });
  assert.equal(get(s.editingGroupId), first, 'the name field opens right away');
  const second = s.createGroup(['b']);
  assert.deepEqual(get(s.tabGroups).members, { a: first, b: second });
  assert.equal(get(s.tabGroups).groups.length, 2);
});

test('a group with no tab left disappears', () => {
  const s = load();
  const group = s.createGroup(['a']);
  s.removeTabsFromGroup(['a']);
  assert.deepEqual(get(s.tabGroups), { groups: [], members: {} });
  const other = s.createGroup(['b']);
  s.addTabsToGroup(['c'], other);
  s.ungroup(other);
  assert.deepEqual(get(s.tabGroups), { groups: [], members: {} });
  assert.notEqual(group, other);
});

test('rename, color, mode and folding change only that group', () => {
  const s = load();
  const one = s.createGroup(['a']), two = s.createGroup(['b']);
  s.renameGroup(one, `  ${'x'.repeat(60)}  `);
  s.setGroupColor(one, 'red');
  s.toggleGroupCollapsed(one);
  const groups = get(s.tabGroups).groups;
  assert.equal(groups[0].name.length, 40);
  assert.equal(groups[0].color, 'red');
  assert.equal(groups[0].collapsed, true);
  s.setGroupMode(one, 'popup');
  assert.equal(get(s.tabGroups).groups[0].collapsed, false, 'switching mode unfolds the group');
  assert.equal(get(s.tabGroups).groups[1].id, two);
  assert.equal(get(s.tabGroups).groups[1].mode, 'collapse');
});

test('nothing is saved before the saved groups were read, and changes are merged after', async () => {
  const saves = [];
  const saved = { groups: [{ id: 'old', name: 'Old', color: 'blue', mode: 'collapse', collapsed: false }], members: { t: 'old' } };
  const s = load({ saved, saves });
  s.createGroup(['early']);
  await new Promise(resolve => setTimeout(resolve, 250));
  assert.equal(saves.length, 0, 'an early change is not written over what was saved');
  await s.loadTabGroups();
  const state = get(s.tabGroups);
  assert.deepEqual(state.groups.map(group => group.id), ['old', 'g1']);
  assert.equal(state.members.t, 'old');
  assert.equal(state.members.early, 'g1');
  s.renameGroup('old', 'Renamed');
  await new Promise(resolve => setTimeout(resolve, 250));
  assert.equal(saves.at(-1).groups[0].name, 'Renamed');
});
