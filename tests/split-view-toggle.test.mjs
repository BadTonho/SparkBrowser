import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import ts from "typescript";

const writable = (initial) => {
  let value = initial;
  const listeners = new Set();
  return {
    set: (next) => { value = next; listeners.forEach((listener) => listener(value)); },
    subscribe: (listener) => { listeners.add(listener); listener(value); return () => listeners.delete(listener); },
  };
};
const get = (store) => { let value; store.subscribe((next) => { value = next; })(); return value; };

function setup(initialTabs) {
  const tabs = writable(initialTabs);
  const split = writable(null);
  const calls = { addTab: 0, setSplitView: [] };
  const imports = {
    "svelte/store": { get },
    "./browserStore": {
      tabs,
      async addTab() {
        calls.addTab += 1;
        await new Promise((resolve) => setTimeout(resolve, 5));
        const id = `new-${calls.addTab}`;
        tabs.set([...get(tabs).map((tab) => ({ ...tab, is_active: false })), { id, is_active: true, workspace_id: "ws" }]);
      },
      applyTabsSnapshot(packet) { split.set(packet.split_layout); },
    },
    "./workspaceSelectionStore": { activeWorkspaceId: writable("ws") },
    "./splitViewStore": { splitLayout: split },
    "../services/bridge/split-view": {
      async setSplitView(left, right) {
        calls.setSplitView.push([left, right]);
        await new Promise((resolve) => setTimeout(resolve, 5));
        return { split_layout: left ? { left_tab_id: left, right_tab_id: right } : null };
      },
    },
    "../services/split-tab-groups.js": {
      pickSplitCompanion: (list, activeId) => list.find((tab) => tab.id !== activeId)?.id ?? null,
    },
    "../services/ignored-error": { logIgnored: (error) => { throw error; } },
  };
  const source = ts.transpileModule(readFileSync("src/lib/stores/splitViewActions.ts", "utf8"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, {
    exports,
    require: (name) => {
      if (!(name in imports)) throw Error("Missing test import: " + name);
      return imports[name];
    },
  });
  return { actions: exports, calls, split, tabs };
}

test("pressing the split shortcut many times while it opens creates only one blank tab", async () => {
  const { actions, calls, split, tabs } = setup([{ id: "a", is_active: true, workspace_id: "ws" }]);
  await Promise.all(Array.from({ length: 6 }, () => actions.toggleSplitView()));
  assert.equal(calls.addTab, 1);
  assert.equal(get(tabs).length, 2);
  assert.deepEqual(calls.setSplitView, [["a", "new-1"]]);
  assert.deepEqual(get(split), { left_tab_id: "a", right_tab_id: "new-1" });
});

test("the next press after a change finishes toggles again", async () => {
  const { actions, calls, split } = setup([
    { id: "a", is_active: true, workspace_id: "ws" },
    { id: "b", is_active: false, workspace_id: "ws" },
  ]);
  await actions.toggleSplitView();
  assert.ok(get(split));
  await actions.toggleSplitView();
  assert.equal(get(split), null);
  assert.equal(calls.addTab, 0);
  assert.deepEqual(calls.setSplitView, [["a", "b"], [null, null]]);
});
