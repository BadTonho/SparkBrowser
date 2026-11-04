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
  const state = read('src-tauri/src/state/browser_state.rs');
  assert.match(state, /workspace_isolation: AtomicBool::new\(false\)/);
  const command = read('src-tauri/src/commands/workspaces.rs');
  assert.match(command, /pub async fn set_workspace_isolation\(/);
  assert.match(command, /preferences_repo::WORKSPACE_ISOLATION_KEY/);
  // Only tabs' WebViews change folders; the shell keeps the default profile.
  assert.match(read('src-tauri/src/window/webview_manager.rs'), /workspace_data::content_directory\(app, tab_id\)/);
  assert.equal(read('src-tauri/src/window/webview_manager.rs').match(/\.data_directory\(/g)?.length, 1);
});

test('isolated data folders are never derived from untrusted workspace ids', () => {
  const workspace_data = read('src-tauri/src/window/workspace_data.rs');
  assert.match(workspace_data, /is_ascii_alphanumeric\(\)/);
  assert.match(workspace_data, /sha2::\{Digest, Sha256\}|use sha2/);
  assert.match(workspace_data, /format!\("h-\{:x\}", Sha256::digest/);
  // Workspace deletion reclaims its folder on a best-effort basis.
  assert.match(read('src-tauri/src/commands/workspaces.rs'), /workspace_data::remove\(&app, &workspace_id\)/);
});

test('moved tabs drop their WebView before joining another workspace', () => {
  const tabs = read('src-tauri/src/commands/tabs.rs');
  assert.match(tabs, /state\s*\.\s*workspace_isolation\s*\.\s*load\(std::sync::atomic::Ordering::Acquire\)/);
  assert.match(tabs, /close_tab_webview\(&app, &tab_id\)/);
});

test('per-navigation and per-close token revocation stays in place', () => {
  const lifecycle = read('src-tauri/src/window/content_lifecycle.rs');
  // Closing or hibernating a WebView retires its authenticated sessions.
  assert.match(lifecycle, /keyboard_bridge_tokens/);
  assert.match(lifecycle, /keyboard_bridge_sequences/);
  assert.match(lifecycle, /super::new_tab::revoke\(app, label\)/);
  // Every committed navigation admits only growing sequences from one document.
  const sequence = read('src-tauri/src/security/sequence.rs');
  assert.match(sequence, /if sequence <= state\.last \{/);
  assert.match(sequence, /state\.retired\.insert/);
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
