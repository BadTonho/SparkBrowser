import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(path, 'utf8');

test('the option is a device-local preference that defaults to on and only "false" turns it off', () => {
  const store = read('src/lib/stores/settingsStore.ts');
  assert.match(store, /spark_hibernate_other_workspaces/);
  assert.match(store, /writable\(safeStorage\.getItem\(HIBERNATE_OTHER_WORKSPACES_KEY\) !== "false"\)/, 'on unless explicitly turned off');
  assert.match(store, /export function setHibernateOtherWorkspaces\(enabled: boolean\)/);
});

test('switching workspace schedules a delayed hibernation and turning the option on runs it once', () => {
  const lifecycle = read('src/lib/stores/workspaceHibernation.ts');
  assert.match(lifecycle, /WORKSPACE_HIBERNATION_DELAY_MS = 3_000/);
  assert.match(lifecycle, /activeWorkspaceId\.subscribe/);
  assert.match(lifecycle, /if \(id === workspace\) return;/, 'the initial value is not a switch');
  assert.match(lifecycle, /clearTimeout\(timer\);\s*timer = setTimeout/, 'a new switch restarts the wait');
  assert.match(lifecycle, /turnedOn/);
  assert.match(lifecycle, /if \(disposed \|\| !get\(hibernateOtherWorkspaces\)\) return;/, 'turning it off cancels pending work');
  assert.match(read('src/App.svelte'), /initializeWorkspaceHibernation\(\)/);
  assert.match(read('src/App.svelte'), /stopWorkspaceHibernation\(\)/);
  assert.match(read('src/lib/services/bridge/navigation.ts'), /invoke<TabSnapshot>\("hibernate_other_workspaces"\)/);
});

test('the backend command is registered, listed in the ACL manifest and allowed for the shell', () => {
  assert.match(read('src-tauri/src/commands/manifest.rs'), /"hibernate_other_workspaces"/);
  assert.match(read('src-tauri/src/lib.rs'), /\bhibernate_other_workspaces,/);
  assert.match(read('src-tauri/capabilities/default.json'), /"allow-hibernate-other-workspaces"/);
  const policy = read('src-tauri/src/state/memory_policy.rs');
  assert.match(policy, /!tab\.protected_from_hibernation\(\)/, 'protected tabs stay awake');
  assert.match(policy, /hibernation_domain_matches\(&tab\.url, exemptions\)/, 'exempt domains stay awake');
});

test('Performance settings list the option, search finds it and isolation reads Isolated or Shared', () => {
  assert.match(read('src/lib/components/settings/PerformanceSettings.svelte'), /id="setting-hibernate-other-workspaces"[\s\S]*?setHibernateOtherWorkspaces\(event\.currentTarget\.checked\)/);
  assert.match(read('src/lib/components/settings/settings-catalog.ts'), /id: "hibernate-other-workspaces"[^\n]*targetId: "setting-hibernate-other-workspaces"/);
  assert.match(read('src/lib/components/settings/WorkspaceSettings.svelte'), /content_isolation_on[\s\S]*content_isolation_off/);
  for (const [language, on, off] of [['en', 'Isolated', 'Shared'], ['pt-BR', 'Isolado', 'Compartilhado']]) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/settings.json`));
    assert.equal(catalog['settings.content_isolation_on'], on);
    assert.equal(catalog['settings.content_isolation_off'], off);
    assert.ok(catalog['settings.hibernate_other_workspaces'] && catalog['settings.hibernate_other_workspaces_description']);
  }
});
