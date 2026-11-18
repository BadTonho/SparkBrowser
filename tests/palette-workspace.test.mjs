import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import test from "node:test";
import ts from "typescript";

// A page whose root remembers its attributes and inline styles, and tells its observer when the palette changes.
function page() {
  const style = new Map();
  const attributes = new Map();
  const observers = [];
  const root = {
    style: { setProperty: (name, value) => style.set(name, value), removeProperty: name => style.delete(name) },
    hasAttribute: name => attributes.has(name),
    setPalette(value) { if (value) attributes.set("data-palette", value); else attributes.delete("data-palette"); observers.forEach(callback => callback()); },
  };
  class MutationObserver { constructor(callback) { observers.push(callback); } observe() {} }
  return { root, style, MutationObserver };
}

function load(env) {
  const source = ts.transpileModule(readFileSync("src/lib/stores/workspaceStore.ts", "utf8"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  const stub = new Proxy({}, { get: () => () => ({ subscribe: () => () => {} }) });
  const modules = {
    "../services/storage.js": { safeStorage: undefined },
    "svelte/store": { writable: initial => ({ set() {}, update() {}, subscribe: callback => { callback(initial); return () => {}; } }), readonly: store => store, get: () => [{ id: "ws", accent_primary: "#e5e5e5", accent_secondary: "#9a9a9a" }] },
    "../services/tauri-bridge": stub,
    "./browserStore": { applyTabsSnapshot() {}, tabs: { subscribe: () => () => {} } },
    "../constants/workspaceDefaults": { DEFAULT_WORKSPACES: [], NEW_WORKSPACE_DEFAULT_COLORS: [] },
    "./workspaceSelectionStore": { activeWorkspaceId: { subscribe: callback => { callback("ws"); return () => {}; }, set() {} }, confirmWorkspaceSelection() {}, workspaceSelectionRevision: () => 0 },
  };
  runInNewContext(source, {
    exports, MutationObserver: env.MutationObserver, console,
    document: { documentElement: env.root },
    require: id => { if (!(id in modules)) throw new Error(`Missing import: ${id}`); return modules[id]; },
  });
  return exports;
}

const workspace = { accent_primary: "#e5e5e5", accent_secondary: "#9a9a9a" };

test("without a palette the workspace colors paint the accent, as before", () => {
  const env = page();
  const store = load(env);
  store.applyWorkspaceTheme(workspace);
  assert.equal(env.style.get("--accent-primary"), "#e5e5e5");
  assert.equal(env.style.get("--accent-gradient"), "linear-gradient(135deg, #e5e5e5 0%, #9a9a9a 100%)");
});

test("a chosen palette replaces the workspace colors, so its attribute rules are not overridden", () => {
  const env = page();
  const store = load(env);
  env.root.setPalette("rose");
  store.applyWorkspaceTheme(workspace);
  for (const name of ["--accent-primary", "--accent-secondary", "--accent-foreground", "--accent-gradient"]) {
    assert.equal(env.style.has(name), false, `${name} is left to the palette`);
  }
});

test("choosing or clearing a palette repaints the accent at once, including from another window", () => {
  const env = page();
  const store = load(env);
  store.applyWorkspaceTheme(workspace);
  assert.equal(env.style.get("--accent-primary"), "#e5e5e5");
  env.root.setPalette("violet");
  assert.equal(env.style.has("--accent-primary"), false);
  env.root.setPalette(null);
  assert.equal(env.style.get("--accent-primary"), "#e5e5e5", "the workspace colors come back with Graphite");
});

test("the module can paint a workspace while it is still loading (the subscription runs at once)", () => {
  const env = page();
  // Loading must not throw, and the accent is already painted when it finishes.
  load(env);
  assert.equal(env.style.get("--accent-primary"), "#e5e5e5");
});
