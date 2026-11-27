import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(path, 'utf8');
const privacy = read('src/lib/components/settings/PrivacySecuritySettings.svelte');

test('privacy and security is split into titled categories, each with its own settings', () => {
  const groups = [...privacy.matchAll(/<div class="setting-subgroup" role="group" aria-labelledby="(privacy-[a-z-]+-heading)">\s*<h3 class="subgroup-title" id="\1">\{\$t\("([^"]+)"\)\}<\/h3>([\s\S]*?)\n  <\/div>/g)];
  const byTitle = Object.fromEntries(groups.map(([, , key, body]) => [key, body]));
  const expected = {
    'settings.network_privacy': ['id="setting-network-doh"', 'id="setting-network-https"'],
    'settings.privacy_group_history': ['id="clear-local-history"', 'id="setting-history-retention"'],
    'settings.privacy_group_safe_browsing': ['id="setting-smart-screen"'],
    'settings.privacy_group_shield': ['id="setting-adblock-shield"', '<ShieldExceptions />', 'id="setting-popup-policy"'],
    'settings.privacy_group_tracking': ['<GpcSettings />', '<TrackingCleaningSettings />'],
  };
  assert.deepEqual(Object.keys(byTitle), Object.keys(expected));
  for (const [key, parts] of Object.entries(expected)) {
    for (const part of parts) assert.ok(byTitle[key].includes(part), `${part} is under ${key}`);
  }
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/settings.json`));
    for (const key of Object.keys(expected)) assert.ok(catalog[key], `${language} ${key}`);
  }
});

test('privacy switches and exception lists use the shared settings styles', () => {
  // Bare switch inputs are drawn by settings.css, which the settings window always has.
  assert.doesNotMatch(privacy, /switch-control|switch-track|privacy-switch-row/);
  for (const id of ['setting-network-https', 'setting-smart-screen', 'setting-popup-policy']) {
    assert.match(privacy, new RegExp(`<label class="setting-row" id="${id}"[^>]*>[\\s\\S]*?<input type="checkbox" role="switch"[^\\n]*/>\\s*</label>`), id);
  }
  for (const file of ['GpcSettings', 'TrackingCleaningSettings', 'ShieldExceptions']) {
    const source = read(`src/lib/components/settings/${file}.svelte`);
    assert.doesNotMatch(source, /<style/, `${file} has no styles of its own`);
    assert.match(source, /class="exception-editor"/, file);
    assert.match(source, /class="exception-list"/, file);
    assert.match(source, /class="pill-action"/, file);
  }
  const css = read('src/lib/components/settings/settings.css');
  for (const rule of ['.setting-card {', '.setting-card-head {', '.exception-editor {', '.exception-list li {', '.settings-message {']) {
    assert.ok(css.includes(rule), rule);
  }
});
