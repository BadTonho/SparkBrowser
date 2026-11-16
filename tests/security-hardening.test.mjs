import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFileSync } from 'node:fs';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('content WebViews harden DevTools and never expose host objects', () => {
  const windows = read('src-tauri/src/security/windows.rs');
  assert.match(windows, /SetAreHostObjectsAllowed\(false\)/);
  assert.match(windows, /SetAreDevToolsEnabled\(tauri::is_dev\(\)\)/);
  // Fail closed: a partially installed boundary shuts the app down.
  assert.match(windows, /security\.initialization_failed/);
  assert.match(windows, /app\.exit\(1\)/);
});

test('workspace isolation is opt-in, durable and scoped to content WebViews', () => {
  const state = read('src-tauri/src/state/domains/policies.rs');
  assert.match(state, /workspace_isolation: AtomicBool::new\(false\)/);
  const command = read('src-tauri/src/state/session_ops/workspaces.rs');
  assert.match(command, /pub async fn set_workspace_isolation\(/);
  assert.match(command, /preferences_repo::WORKSPACE_ISOLATION_KEY/);
  // Only tabs' WebViews change folders; the shell keeps the default profile.
  assert.match(read('src-tauri/src/window/content_lifecycle.rs'), /workspace_data::content_directory\(app, id\)/);
  assert.equal(read('src-tauri/src/window/content_factory.rs').match(/\.data_directory\(/g)?.length, 1);
});

test('workspace deletion is wired to reclaim its isolated folder', () => {
  // Folder safety and identity separation are executed by workspace_data Rust tests.
  // Workspace deletion reclaims its folder on a best-effort basis.
  assert.match(read('src-tauri/src/state/session_ops/workspaces.rs'), /workspace_data::remove\(&app, &workspace_id\)/);
});

test('workspace moves recreate content only when the resolved profile changes', () => {
  const lifecycle = read('src-tauri/src/window/content_lifecycle.rs');
  assert.match(lifecycle, /content_profiles\(\)\.matches|content_profiles\(\)[\s\S]*\.matches/);
  assert.match(lifecycle, /close_and_clean\(app, &label\)/);
});

test('per-navigation and per-close token revocation stays in place', () => {
  const lifecycle = read('src-tauri/src/window/content_lifecycle.rs');
  // Closing or hibernating a WebView retires its authenticated sessions.
  assert.match(lifecycle, /keyboard_bridge_tokens/);
  assert.match(lifecycle, /keyboard_bridge_sequences/);
  assert.match(lifecycle, /super::new_tab::revoke\(app, label\)/);
  // Replay/retired-document rejection is executed by security::sequence Rust tests.
});

test('isolation preferences flow through backups and the settings UI', () => {
  const capability = JSON.parse(read('src-tauri/capabilities/default.json'));
  assert.ok(capability.permissions.includes('allow-get-workspace-isolation'));
  assert.ok(capability.permissions.includes('allow-set-workspace-isolation'));
  const manifest = read('src-tauri/src/commands/manifest.rs');
  assert.match(manifest, /"get_workspace_isolation"/);
  assert.match(manifest, /"set_workspace_isolation"/);
  const model = read('src-tauri/src/backup/model.rs');
  assert.match(model, /pub workspace_isolation: bool/);
  const preferences = read('src/lib/services/backup/preferences.ts');
  assert.match(preferences, /workspace_isolation:get\(workspaceIsolation\)/);
  assert.match(preferences, /setWorkspaceIsolationEnabled\(p\.workspace_isolation\?\?false\)/);
  const settings = read('src/lib/components/settings/WorkspaceSettings.svelte');
  assert.match(settings, /setting-workspace-isolation/);
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/settings.json`));
    assert.ok(catalog['settings.content_isolation_9f15']);
    assert.ok(catalog['settings.content_isolation_desc_fea8']);
  }
});
