import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import test from "node:test";
import ts from "typescript";

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const plain = value => JSON.parse(JSON.stringify(value));
const DAY = 24 * 60 * 60 * 1000;

function loadStore({ saved = null, cleaning = async () => ({ done: [], failed: [] }), reports = [] } = {}) {
  const source = ts.transpileModule(read("src/lib/stores/autoCleanStore.ts"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const written = [];
  const exports = {};
  const writable = initial => { const box = { value: initial }; return { set: next => { box.value = next; }, subscribe: () => () => {}, get box() { return box; } }; };
  runInNewContext(source, {
    exports, setTimeout, clearTimeout, setInterval, clearInterval, JSON, Date, Object, Number, String, Array, Boolean,
    require: id => ({
      "svelte/store": { writable, get: store => store.box.value },
      "../services/storage.js": { safeStorage: { getItem: () => saved, setItem: (key, value) => written.push([key, value]) } },
      "../services/bridge/runtime": { isTauriRuntime: () => true, invoke: async (name, args) => { reports.push([name, args]); } },
      "../services/ignored-error": { logIgnored() {} },
      "../services/cleaning": { CLEAN_ITEMS: ["cache", "site", "history", "downloads"], runCleaning: cleaning },
    })[id],
  });
  return { ...exports, written };
}

test("the automatic cleaning is off by default and a damaged value never turns it on", () => {
  const { autoClean, normalizeAutoClean } = loadStore();
  assert.equal(autoClean.box.value.enabled, false);
  const items = { cache: true, site: false, history: false, downloads: false };
  assert.equal(normalizeAutoClean({ enabled: true, items, days: 10 }).enabled, true);
  assert.equal(normalizeAutoClean({ enabled: true, items, days: 0 }).enabled, false, "no period");
  assert.equal(normalizeAutoClean({ enabled: true, items, days: 7 }).enabled, false, "a period that is not offered");
  assert.equal(normalizeAutoClean({ enabled: true, items: { cache: false }, days: 10 }).enabled, false, "nothing to clear");
  assert.equal(normalizeAutoClean({ enabled: "yes", items, days: 10 }).enabled, false);
  for (const damaged of [null, undefined, "x", 7, []]) assert.equal(normalizeAutoClean(damaged).enabled, false);
});

test("it is due only when on and not run in the last day", () => {
  const { isCleaningDue, normalizeAutoClean } = loadStore();
  const on = { enabled: true, items: { cache: true }, days: 10, lastRun: 1000 };
  assert.equal(isCleaningDue(normalizeAutoClean(on), 1000 + DAY - 1), false);
  assert.equal(isCleaningDue(normalizeAutoClean(on), 1000 + DAY), true);
  assert.equal(isCleaningDue(normalizeAutoClean({ ...on, lastRun: 0 }), 5 * DAY), true, "never run");
  assert.equal(isCleaningDue(normalizeAutoClean({ ...on, enabled: false }), 5 * DAY), false);
});

test("it clears what was chosen with the saved period, saves the time first, and reports what failed", async () => {
  const calls = [];
  const reports = [];
  const saved = JSON.stringify({ enabled: true, items: { cache: true, history: true }, days: 20, lastRun: 0 });
  const { runAutoCleanIfDue, autoClean } = loadStore({
    saved, reports,
    cleaning: async (items, days) => { calls.push([plain(items), days, autoClean.box.value.lastRun]); return { done: ["cache"], failed: [{ items: ["history"], error: new Error("locked") }] }; },
  });
  assert.equal(await runAutoCleanIfDue(5 * DAY), true);
  assert.deepEqual(calls, [[{ cache: true, site: false, history: true, downloads: false }, 20, 5 * DAY]], "the time was saved before the cleaning started");
  assert.equal(reports.length, 1);
  assert.equal(reports[0][0], "report_frontend_error");
  assert.match(reports[0][1].summary, /history.*locked/);
  assert.equal(reports[0][1].level, "warning");
  assert.equal(await runAutoCleanIfDue(5 * DAY + 1000), false, "not again the same day");
});

test("turning it on with no period saves it off", () => {
  const { setAutoClean, autoClean, written } = loadStore();
  setAutoClean({ enabled: true, items: { cache: true, site: false, history: false, downloads: false }, days: 0 });
  assert.equal(autoClean.box.value.enabled, false);
  setAutoClean({ enabled: true, days: 30 });
  assert.equal(autoClean.box.value.enabled, true);
  assert.equal(JSON.parse(written.at(-1)[1]).days, 30);
});
