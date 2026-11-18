import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import test from "node:test";
import ts from "typescript";

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const plain = value => JSON.parse(JSON.stringify(value));

function load(path, imports = {}) {
  const source = ts.transpileModule(read(path), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, { exports, require: id => { if (!(id in imports)) throw new Error(`Missing import: ${id}`); return imports[id]; } });
  return exports;
}

const tab = (id, extra = {}) => ({ id, title: id, url: `https://${id}.test/`, workspace_id: "w", is_active: false, is_hibernated: false, ...extra });

test("the tabs hibernated first are the ones used longest ago, never the active, visible or busy ones", () => {
  const { hibernationCandidates } = load("src/lib/services/resource-limits.ts");
  const ids = list => hibernationCandidates(list).map(item => item.id);
  assert.deepEqual(ids([
    tab("recent", { last_active_epoch: 300 }),
    tab("old", { last_active_epoch: 100 }),
    tab("never"),
    tab("active", { is_active: true, last_active_epoch: 1 }),
    tab("asleep", { is_hibernated: true, last_active_epoch: 1 }),
    tab("split", { is_visible: true, last_active_epoch: 1 }),
    tab("typing", { activity: { dirty_form: true, playing_media: false, downloading: false }, last_active_epoch: 1 }),
    tab("music", { activity: { dirty_form: false, playing_media: true, downloading: false }, last_active_epoch: 1 }),
    tab("saving", { activity: { dirty_form: false, playing_media: false, downloading: true }, last_active_epoch: 1 }),
    tab("pip", { activity: { dirty_form: false, playing_media: false, downloading: false, picture_in_picture: true }, last_active_epoch: 1 }),
  ]), ["never", "old", "recent"]);
});

test("memory counts the open tabs and only goes over the limit with a margin", () => {
  const { usedMegabytes, isOverLimit } = load("src/lib/services/resource-limits.ts");
  assert.equal(usedMegabytes({ a: 100 * 1024 * 1024, b: 300 * 1024 * 1024, c: 0, d: -5 }), 400);
  assert.equal(isOverLimit(2048, 0), false, "no limit is never over");
  assert.equal(isOverLimit(2050, 2048), false, "a hair over is not enough");
  assert.equal(isOverLimit(2200, 2048), true);
});

test("speeds and limits are written the way people read them", () => {
  const { formatSpeed, formatMemoryLimit } = load("src/lib/services/resource-limits.ts");
  assert.equal(formatSpeed(0), "0 kbps");
  assert.equal(formatSpeed(42_500), "340 kbps");
  assert.equal(formatSpeed(937_500), "7.5 Mbps");
  assert.equal(formatSpeed(1_562_500), "13 Mbps", "from 10 Mbps up the decimals are dropped");
  assert.equal(formatSpeed(12_500_000), "100 Mbps");
  assert.equal(formatMemoryLimit(2048), "2 GB");
  assert.equal(formatMemoryLimit(512), "512 MB");
});

test("a saved limit that no slider position can give means no limit, and the slider maps both ways", () => {
  const { normalizeLimits, LIMIT_RANGES, limitToSlider, sliderToLimit } = load("src/lib/stores/resourceLimitsStore.ts", {
    "svelte/store": { get: () => ({}), writable: initial => ({ set() {}, subscribe: () => () => {}, value: initial }) },
    "../services/storage.js": { safeStorage: { getItem: () => null, setItem() {} } },
    "../services/bridge/runtime": { invoke: async () => {}, isTauriRuntime: () => false },
    "../services/ignored-error": { logIgnored() {} },
  });
  assert.deepEqual(plain(normalizeLimits({ ramMb: 2048, cpuPercent: 50, networkKbps: 5000 })), { ramMb: 2048, cpuPercent: 50, networkKbps: 5000 });
  assert.deepEqual(plain(normalizeLimits({ ramMb: 3072, cpuPercent: 15, networkKbps: 25000 })), { ramMb: 3072, cpuPercent: 15, networkKbps: 25000 });
  assert.deepEqual(plain(normalizeLimits({ ramMb: 100, cpuPercent: 120, networkKbps: 50 })), { ramMb: 0, cpuPercent: 0, networkKbps: 0 }, "out of range");
  const range = LIMIT_RANGES.ramMb;
  const last = range.values.length;
  assert.equal(sliderToLimit(range, last), 0, "the last position is no limit");
  assert.equal(limitToSlider(range, 0), last);
  assert.equal(sliderToLimit(range, range.values.indexOf(2048)), 2048);
  assert.equal(limitToSlider(range, 2048), range.values.indexOf(2048));
  const net = LIMIT_RANGES.networkKbps.values;
  assert.deepEqual([net[0], net[8], net[9], net[16], net[17], net[21], net[22], net.at(-1)], [1000, 9000, 10000, 45000, 50000, 90000, 100000, 1000000], "fine steps first, coarse ones after");
  assert.deepEqual(plain(normalizeLimits({ networkKbps: 500000 })), { ramMb: 0, cpuPercent: 0, networkKbps: 500000 });
  assert.deepEqual(plain(normalizeLimits({ ramMb: 1536, cpuPercent: 33, networkKbps: 37500 })), { ramMb: 1536, cpuPercent: 33, networkKbps: 37500 }, "a typed value between two steps is kept");
  assert.equal(limitToSlider(LIMIT_RANGES.ramMb, 1500), LIMIT_RANGES.ramMb.values.indexOf(1536), "the thumb sits on the nearest step");
  assert.deepEqual(plain(normalizeLimits({ ramMb: 1.5, networkKbps: 1_000_001 })), { ramMb: 0, cpuPercent: 0, networkKbps: 0 });
  assert.deepEqual(plain(normalizeLimits({ ramMb: 1, cpuPercent: 99, networkKbps: -1 })), { ramMb: 0, cpuPercent: 0, networkKbps: 0 });
  for (const damaged of [null, undefined, "x", 7, [], { ramMb: "2048" }]) assert.deepEqual(plain(normalizeLimits(damaged)), { ramMb: 0, cpuPercent: 0, networkKbps: 0 });
});

test("the controller hibernates one candidate per check, tries the next when one is protected, and does nothing without a limit", async () => {
  const state = { limits: { ramMb: 1024 }, tabs: [tab("a", { last_active_epoch: 1 }), tab("b", { last_active_epoch: 2 }), tab("c", { last_active_epoch: 3 })], memory: { a: 900 * 1024 * 1024, b: 900 * 1024 * 1024 } };
  const calls = [];
  const get = store => store.value();
  const stores = {
    resourceLimits: { value: () => state.limits },
    tabs: { value: () => state.tabs },
    tabsMemory: { value: () => state.memory },
  };
  const { hibernationCandidates, usedMegabytes, isOverLimit } = load("src/lib/services/resource-limits.ts");
  const { enforceMemoryLimit } = load("src/lib/stores/resourceLimitsController.ts", {
    "svelte/store": { get },
    "./browserStore": { tabs: stores.tabs, suspendTab: async id => { calls.push(id); if (id === "a") throw new Error("protected"); } },
    "./tabMemoryStore": { tabsMemory: stores.tabsMemory, refreshTabsMemory: async () => {} },
    "./resourceLimitsStore": { resourceLimits: stores.resourceLimits, applyNativeLimits: async () => {} },
    "../services/resource-limits": { hibernationCandidates, usedMegabytes, isOverLimit },
    "../services/bridge/runtime": { isTauriRuntime: () => true },
    "../services/ignored-error": { logIgnored() {} },
  });
  assert.equal(await enforceMemoryLimit(), true);
  assert.deepEqual(calls, ["a", "b"], "a protected tab is skipped, then one tab is hibernated and the check ends");
  calls.length = 0;
  state.limits = { ramMb: 0 };
  assert.equal(await enforceMemoryLimit(), false);
  assert.deepEqual(calls, []);
  state.limits = { ramMb: 4096 };
  assert.equal(await enforceMemoryLimit(), false, "under the limit nothing is hibernated");
});

test("the limits are registered end to end and the processor cap leaves the interface outside", () => {
  for (const file of ["src-tauri/src/lib.rs", "src-tauri/src/commands/manifest.rs", "src-tauri/capabilities/default.json"]) {
    for (const name of ["set_resource_limits", "set_resource_monitor", "get_resource_usage"]) {
      assert.ok(read(file).includes(name) || read(file).includes(name.replaceAll("_", "-")), `${file} ${name}`);
    }
  }
  const windows = read("src-tauri/src/resources/windows.rs");
  assert.ok(windows.includes("family.remove(&own)"), "the browser's own process is never capped");
  assert.ok(read("src-tauri/src/window/content_factory.rs").includes("resources::webview_created"), "pages opened later get the limits too");
  assert.ok(read("src-tauri/src/downloads/transfer.rs").includes("throttle(started, received, cancel)"), "the browser's own downloads follow the speed limit");
});
