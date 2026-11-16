import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

const read = path => readFileSync(path, 'utf8');
function load(path) {
  const exports = {};
  runInNewContext(ts.transpileModule(read(path), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText, { exports, Number, String, Map, Object, Array, RegExp });
  return exports;
}
const { parseInline, parseChangelog, compareVersions, changelogEntries } = load('src/lib/services/changelog.ts');
const plain = value => JSON.parse(JSON.stringify(value));

test('inline bold and code become spans and everything else stays literal text', () => {
  assert.deepEqual(plain(parseInline('Open **Music and video** then `youtube-nocookie.com` <b>x</b>')), [
    { text: 'Open ' }, { text: 'Music and video', bold: true }, { text: ' then ' }, { text: 'youtube-nocookie.com', code: true }, { text: ' <b>x</b>' },
  ]);
  assert.deepEqual(plain(parseInline('no markup')), [{ text: 'no markup' }]);
});

test('a changelog file is read into a title, a note and ordered sections of items', () => {
  const entry = parseChangelog('# Spark 0.2.0\r\n\r\nIn progress.\r\n\r\n## New features\r\n\r\n- First **big** thing\r\n  continued here.\r\n\r\n- Second\r\n\r\n## Fixes\r\n- A fix\r\n', '0.2.0');
  assert.equal(entry.title, 'Spark 0.2.0');
  assert.equal(entry.note, 'In progress.');
  assert.deepEqual(plain(entry.sections.map(section => [section.heading, section.items.length])), [['New features', 2], ['Fixes', 1]]);
  assert.equal(entry.sections[0].items[0].map(part => part.text).join(''), 'First big thing continued here.');
  assert.equal(parseChangelog('', '1.0.0').title, 'Spark 1.0.0');
});

test('versions sort numerically and a pre-release ranks below its release', () => {
  const sorted = ['0.9.0', '0.10.0', '1.0.0-beta.1', '1.0.0', '0.1.1'].sort(compareVersions).reverse();
  assert.deepEqual(plain(sorted), ['1.0.0', '1.0.0-beta.1', '0.10.0', '0.9.0', '0.1.1']);
});

test('entries are newest first, use the chosen language and fall back to English', () => {
  const files = {
    '/changelog/0.1.1/en.md': '# Spark 0.1.1\n\n## New features\n- English one\n',
    '/changelog/0.1.1/pt-BR.md': '# Spark 0.1.1\n\n## Novas funcionalidades\n- Português um\n',
    '/changelog/0.2.0/en.md': '# Spark 0.2.0\n\n## Fixes\n- Only English\n',
    '/changelog/README.md': '# ignored',
    '/other/0.3.0/en.md': '# ignored',
  };
  const portuguese = changelogEntries(files, 'pt-BR');
  assert.deepEqual(plain(portuguese.map(entry => entry.version)), ['0.2.0', '0.1.1']);
  assert.equal(portuguese[1].sections[0].heading, 'Novas funcionalidades');
  assert.equal(portuguese[0].sections[0].heading, 'Fixes', 'falls back to English');
  assert.equal(changelogEntries(files, 'en')[1].sections[0].heading, 'New features');
  assert.deepEqual(plain(changelogEntries({}, 'en')), []);
});

test('every shipped changelog file parses into the three documented categories', () => {
  const versions = readdirSync('changelog', { withFileTypes: true }).filter(item => item.isDirectory()).map(item => item.name);
  assert.ok(versions.length > 0);
  const files = Object.fromEntries(versions.flatMap(version => ['en', 'pt-BR'].map(language => [`/changelog/${version}/${language}.md`, read(`changelog/${version}/${language}.md`)])));
  for (const language of ['en', 'pt-BR']) {
    const entries = changelogEntries(files, language);
    assert.equal(entries.length, versions.length, language);
    for (const entry of entries) assert.ok(entry.sections.every(section => section.items.length > 0), `${language} ${entry.version}`);
  }
});

test('the settings window lists the changelog as a category, a search result and a component', () => {
  assert.match(read('src/lib/stores/settingsStore.ts'), /\| "changelog";/);
  assert.match(read('src/lib/stores/settingsStore.ts'), /"diagnostics",\s*"changelog",\s*\]/);
  const catalog = read('src/lib/components/settings/settings-catalog.ts');
  assert.match(catalog, /\{ id: "changelog", icon: "books", label: "Changelog" \}/);
  assert.match(catalog, /id: "changelog", title: "Changelog"[^\n]*targetId: "changelog-heading"/);
  const modal = read('src/lib/components/settings/SettingsModal.svelte');
  assert.match(modal, /\$activeSettingsCategory === "changelog"\}\s*<ChangelogSettings \/>/);
  const component = read('src/lib/components/settings/ChangelogSettings.svelte');
  assert.ok(!component.includes('{@html'), 'release notes are rendered as text, never as HTML');
  assert.match(component, /id="changelog-heading"/);
  assert.match(read('src/lib/services/changelog-files.ts'), /import\.meta\.glob\("\/changelog\/\*\/\*\.md", \{ query: "\?raw"/);
  for (const language of ['en', 'pt-BR']) {
    const catalogText = JSON.parse(read(`src/lib/i18n/catalogs/${language}/settings.json`));
    for (const key of ['settings.changelog_5f3a', 'settings.changelog_intro_8c1d', 'settings.changelog_empty_2e7b', 'settings.changelog_running_a94c', 'settings.changelog_search_d41e']) assert.ok(catalogText[key], `${language} ${key}`);
  }
  assert.equal(JSON.parse(read('src/lib/i18n/catalogs/en/settings.json'))['settings.changelog_search_d41e'], 'Read what is new and what was fixed in each version of Spark.', 'the search description is translatable by its English text');
});
