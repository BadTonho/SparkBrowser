import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const read = (path) => readFileSync(path, 'utf8').replace(/\r\n/g, '\n');
function loadRows() {
  const js = ts.transpileModule(read('src/lib/components/history/history-rows.ts'), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
  const module = { exports: {} };
  new Function('module', 'exports', js)(module, module.exports);
  return module.exports;
}
const { historyRows, GAP_SECONDS } = loadRows();
const keyOf = (url) => { try { return new URL(url).hostname.replace(/^www\./, ''); } catch { return null; } };
const at = (y, m, d, h, min) => Math.floor(new Date(y, m - 1, d, h, min).getTime() / 1000);

test('visits are grouped as today, yesterday or their date, newest first', () => {
  const now = new Date(2026, 10, 27, 18, 0).getTime();
  const rows = historyRows([
    { id: 1, title: 'A', url: 'https://www.youtube.com/watch?v=x', visited_at: at(2026, 11, 27, 17, 47) },
    { id: 2, title: 'B', url: 'https://github.com/a/b', visited_at: at(2026, 11, 26, 23, 59) },
    { id: 3, title: 'C', url: 'spark://settings', visited_at: at(2026, 11, 20, 10, 0) },
  ], now, keyOf);
  assert.deepEqual(rows.map((row) => row.relative), ['today', 'yesterday', null]);
  assert.equal(rows[0].host, 'youtube.com');
  assert.equal(rows[0].site, 'youtube.com');
  assert.notEqual(rows[1].dayKey, rows[2].dayKey);
});

test('a pause of more than half an hour on the same day is marked, a new day is not', () => {
  const now = new Date(2026, 10, 27, 18, 0).getTime();
  const rows = historyRows([
    { id: 1, title: 'A', url: 'https://a.test', visited_at: at(2026, 11, 27, 17, 34) },
    { id: 2, title: 'B', url: 'https://b.test', visited_at: at(2026, 11, 27, 17, 18) },
    { id: 3, title: 'C', url: 'https://c.test', visited_at: at(2026, 11, 27, 16, 38) },
    { id: 4, title: 'D', url: 'https://d.test', visited_at: at(2026, 11, 26, 9, 0) },
  ], now, keyOf);
  assert.equal(GAP_SECONDS, 1800);
  assert.deepEqual(rows.map((row) => row.gap), [false, false, true, false]);
});

test('single visits can be deleted, and the panel accepts more than one page', () => {
  assert.match(read('src-tauri/src/commands/manifest.rs'), /"delete_history_entries"/);
  assert.match(read('src-tauri/capabilities/default.json'), /"allow-delete-history-entries"/);
  assert.match(read('src-tauri/src/lib.rs'), /delete_history_entries,\n/);
  const overlay = read('src-tauri/src/window/history_overlay/mod.rs');
  assert.match(overlay, /const MAX_ENTRIES: usize = 1200;/, '"Load more" is no longer refused after 60 visits');
  assert.match(read('src-tauri/src/window/history_overlay/bridge.rs'), /"open-many"\s*\|\s*"delete"\s*\|\s*"more-from-site"/);
});

test('the panel draws one card per day with a selection bar and a menu per visit', () => {
  const page = read('public/history-overlay.html');
  for (const id of ['selection-bar', 'selected-count', 'open-selected', 'delete-selected', 'menu']) assert.match(page, new RegExp(`id="${id}"`));
  const script = read('public/overlays/history-overlay.js');
  assert.match(script, /card\.className = 'day'/);
  assert.match(script, /send\('delete', \{ entryIds: \[\.\.\.selected\] \}\)/);
  assert.match(script, /icon\.startsWith\('data:image\/'\)/, 'only icons Spark sent are shown');
  for (const lang of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${lang}/history.json`));
    for (const key of ['day_today', 'day_yesterday', 'selected_count', 'open_selected', 'delete_selected', 'more_from_site', 'delete_entry']) assert.ok(catalog[`history.${key}`], `${key} in ${lang}`);
  }
});
