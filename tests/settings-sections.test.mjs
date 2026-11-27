import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync } from 'node:fs';

const read = path => readFileSync(path, 'utf8');
const modal = read('src/lib/components/settings/SettingsModal.svelte');
const sections = read('src/lib/components/settings/settings-sections.ts');

test('every settings category is a section read on demand, and none is bundled with the modal', () => {
  const union = /export type SettingsCategory =([^;]+);/.exec(read('src/lib/stores/settingsStore.ts'))[1];
  const categories = [...union.matchAll(/"([a-z_]+)"/g)].map(match => match[1]);
  assert.ok(categories.length >= 13);
  for (const category of categories) {
    const loader = new RegExp(`\\b${category}: \\(\\) => import\\("(\\./[^"]+\\.svelte)"\\)`).exec(sections);
    assert.ok(loader, `${category} has a loader`);
    assert.ok(existsSync(`src/lib/components/settings/${loader[1].slice(2)}`), `${loader[1]} exists`);
  }
  // A static import of a section would put it back into the modal's chunk.
  assert.doesNotMatch(modal, /^\s*import \w+Settings from "\.\/[^"]*Settings\.svelte";/m);
});

test('a search result waits for its section before looking for the setting to scroll to', () => {
  const select = /async function selectSearchResult[\s\S]*?\n\}/.exec(modal)[0];
  const waits = select.indexOf('await showSection(entry.category)');
  const looks = select.indexOf('document.getElementById(entry.targetId)');
  assert.ok(waits > 0 && looks > waits, 'showSection is awaited before getElementById');
  // The previous section stays until the new one is ready, and only the newest choice is shown.
  assert.match(modal, /if \(\$activeSettingsCategory !== category\) return false;/);
  assert.match(sections, /pending\.catch\(\(\) => loading\.delete\(category\)\)/, 'a failed read is tried again');
});
