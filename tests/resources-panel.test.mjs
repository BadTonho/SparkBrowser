import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import test from "node:test";
import ts from "typescript";

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

function loadBrowserData(env) {
  const source = ts.transpileModule(read("src/lib/services/browser-data.ts"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, {
    exports, localStorage: env.storage,
    require: id => ({
      "./bridge/runtime": { invoke: env.invoke, isTauriRuntime: () => env.native ?? true },
      "./ignored-error": { logIgnored() {} },
    })[id],
  });
  return exports;
}

const plain = value => JSON.parse(JSON.stringify(value));
const storage = initial => {
  const values = new Map(Object.entries(initial));
  return {
    get length() { return values.size; },
    key: index => [...values.keys()][index] ?? null,
    getItem: key => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, String(value)),
    clear: () => values.clear(),
    values,
  };
};

test("clearing site data keeps the browser's own settings, which live in the same storage", async () => {
  const local = storage({ spark_color_palette: "rose", spark_titlebar_actions_shown: "[\"settings\"]", "site-login": "token" });
  const calls = [];
  const { clearEngineData } = loadBrowserData({ storage: local, invoke: async (command, args) => { calls.push({ command, args }); local.clear(); return { cleared: 1, failed: 0 }; } });
  const report = await clearEngineData({ cache: true, siteData: true });
  assert.deepEqual(plain(report), { cleared: 1, failed: 0 });
  assert.deepEqual(plain(calls), [{ command: "clear_browser_data", args: { request: { cache: true, site_data: true } } }]);
  // The engine emptied the storage; the settings came back, and nothing else was invented.
  assert.equal(local.getItem("spark_color_palette"), "rose");
  assert.equal(local.getItem("spark_titlebar_actions_shown"), "[\"settings\"]");
});

test("clearing only the cache never touches the storage, and a failure still gives the settings back", async () => {
  const local = storage({ spark_color_palette: "rose" });
  const { clearEngineData } = loadBrowserData({ storage: local, invoke: async () => ({ cleared: 1, failed: 0 }) });
  await clearEngineData({ cache: true, siteData: false });
  assert.equal(local.getItem("spark_color_palette"), "rose");

  const failing = storage({ spark_color_palette: "amber" });
  const { clearEngineData: clearFailing } = loadBrowserData({ storage: failing, invoke: async () => { failing.clear(); throw new Error("engine refused"); } });
  await assert.rejects(clearFailing({ cache: false, siteData: true }), /engine refused/);
  assert.equal(failing.getItem("spark_color_palette"), "amber", "the settings are restored even when the engine fails");
});

test("outside the app nothing is invoked", async () => {
  let called = false;
  const { clearEngineData } = loadBrowserData({ storage: storage({}), native: false, invoke: async () => { called = true; } });
  assert.deepEqual(plain(await clearEngineData({ cache: true, siteData: true })), { cleared: 0, failed: 0 });
  assert.equal(called, false);
});

test("the resources button is off by default and sits with the other optional buttons", () => {
  const store = read("src/lib/stores/titlebarActionsStore.ts");
  assert.ok(store.includes('"resources"'));
  const shown = store.match(/DEFAULT_SHOWN_ACTIONS[^=]*= (\[[^\]]*\])/)[1];
  assert.equal(shown.includes("resources"), false, "it is turned on from the quick settings, not shown by default");
  assert.ok(read("src/lib/components/settings/SettingsQuickMenu.svelte").includes('resources: "settings.toolbar_resources"'));
  assert.ok(read("src/lib/components/titlebar/TitlebarActions.svelte").includes('data-action="resources"'));
});

test("the panel is registered end to end and offers the four cleaning choices with a confirmation", () => {
  const rust = read("src-tauri/src/window/panel_popover/mod.rs");
  assert.ok(rust.includes('"resources-popover"') && rust.includes('"resources" => warm::<Resources>'));
  for (const file of ["src-tauri/src/lib.rs", "src-tauri/src/commands/manifest.rs", "src-tauri/capabilities/default.json"]) {
    assert.match(read(file), /set_resources_popover|allow-set-resources-popover/, file);
    assert.match(read(file), /clear_browser_data|allow-clear-browser-data/, file);
  }
  const panel = read("src/lib/components/resources/ResourcesPanel.svelte");
  for (const choice of ["cache", "site", "history", "downloads"]) assert.ok(panel.includes(`id: "${choice}"`), choice);
  assert.ok(panel.includes("confirming") && panel.includes("resources.confirm"), "nothing is cleared on the first click");
  assert.ok(read("src/main.ts").includes("ResourcesSurface.svelte"));
});

test("both languages have every text of the panel", () => {
  const keys = text => Object.keys(JSON.parse(text)).filter(key => key.startsWith("resources.") || key === "settings.toolbar_resources").sort();
  const en = keys(read("src/lib/i18n/catalogs/en/settings.json")), pt = keys(read("src/lib/i18n/catalogs/pt-BR/settings.json"));
  assert.deepEqual(en, pt);
  assert.ok(en.length >= 20);
});
