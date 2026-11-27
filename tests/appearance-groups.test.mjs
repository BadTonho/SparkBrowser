import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(path, 'utf8');
const appearance = read('src/lib/components/settings/AppearanceSettings.svelte');

test('the appearance section is split into titled categories, each with its own settings', () => {
  const groups = [...appearance.matchAll(/<div class="setting-subgroup" role="group" aria-labelledby="(appearance-[a-z]+-heading)">\s*<h3 class="subgroup-title" id="\1">\{\$t\("([^"]+)"\)\}<\/h3>([\s\S]*?)\n  <\/div>/g)];
  const byTitle = Object.fromEntries(groups.map(([, , key, body]) => [key, body]));
  const expected = {
    'settings.appearance_group_language': ['settings-display', 'whatsapp-display-mode'],
    'settings.appearance_group_theme': ['theme-mode', 'color-palette'],
    'settings.appearance_group_surface': ['window-surface-style', 'glass-variant', 'window-surface-opacity', 'window-surface-blur', 'background-blur'],
    'settings.appearance_group_layout': ['browser-layout', 'workspace-switcher-display', 'bookmarks-bar', 'zen-mode'],
    'settings.toolbar_appearance': ['wide-address-bar', 'full-urls', 'forced-dark-pages', 'compact-toolbar', 'rounded-content-frame'],
  };
  assert.deepEqual(Object.keys(byTitle), Object.keys(expected));
  for (const [key, ids] of Object.entries(expected)) {
    for (const id of ids) assert.match(byTitle[key], new RegExp(`id="setting-${id}"`), `${id} is under ${key}`);
  }
  assert.match(byTitle['settings.appearance_group_language'], /<LanguageSettings \/>[\s\S]*<PipAppearanceSettings \/>/);
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/settings.json`));
    for (const key of Object.keys(expected)) assert.ok(catalog[key], `${language} ${key}`);
  }
});

test('toolbar switches are ordinary setting cards drawn by the shared settings styles', () => {
  // Styles of the section itself are not needed: the shared switch style covers a bare switch input.
  assert.doesNotMatch(appearance, /<style/);
  assert.doesNotMatch(appearance, /visual-switch|visual-toggle-row/);
  for (const id of ['wide-address-bar', 'full-urls', 'forced-dark-pages', 'compact-toolbar', 'rounded-content-frame']) {
    assert.match(appearance, new RegExp(`<label class="setting-row" id="setting-${id}"[^>]*>[\\s\\S]*?<input type="checkbox" role="switch"[^\\n]*/>\\s*</label>`), id);
  }
  const css = read('src/lib/components/settings/settings.css');
  assert.match(css, /input\[type="checkbox"\]\[role="switch"\]:not\(:has\(\+ \[aria-hidden="true"\]\)\)/);
  assert.match(css, /\.setting-subgroup \+ \.setting-subgroup/);
  assert.match(css, /\.subgroup-title::after/);
});

test('the settings window loads every section, so their own styles are there when it draws them', () => {
  const surface = read('src/lib/components/settings/SettingsSurface.svelte');
  assert.match(surface, /import \{ preloadAllSections \} from "\.\/settings-sections";/);
  assert.match(surface, /void preloadAllSections\(\)\.then\(/);
  assert.match(surface, /logIgnored\(result\.reason\)/);
  assert.match(read('src/lib/components/settings/settings-sections.ts'), /export function preloadAllSections\(\)[\s\S]*Promise\.allSettled/);
});
