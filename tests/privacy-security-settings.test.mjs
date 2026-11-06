import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('privacy and security controls are searchable and rendered in their own settings category', () => {
  const catalog = read('src/lib/components/settings/settings-catalog.ts');
  const modal = read('src/lib/components/settings/SettingsModal.svelte');
  for (const id of ['privacy_security', 'clear-local-history', 'history-retention', 'popup-setting', 'smart-screen', 'site-permissions', 'adblock-shield']) {
    assert.match(catalog, new RegExp(`id: "${id}"`));
  }
  assert.match(modal, /activeSettingsCategory === "privacy_security"/);
  assert.match(modal, /<PrivacySecuritySettings\s*\/>/);
});

test('local history retention and pop-up choices persist through shell-only native commands', () => {
  const bridge = read('src/lib/services/bridge/privacy-settings.ts');
  const native = read('src-tauri/src/commands/privacy_settings.rs');
  const state = read('src-tauri/src/state/browser_state.rs');
  for (const option of ['one_month', 'three_months', 'six_months', 'twelve_months', 'always']) assert.match(bridge, new RegExp(option));
  assert.match(native, /HISTORY_RETENTION_KEY/);
  assert.match(native, /ALLOW_POPUPS_KEY/);
  assert.match(state, /history_retention_months[\s\S]*AtomicU64::new\(3\)/);
  assert.match(state, /allow_popups: AtomicBool[\s\S]*AtomicBool::new\(false\)/);
});

test('history cleanup deletes history, derived icons and closed tabs, and expiry runs on visit and preference changes', () => {
  const history = read('src-tauri/src/database/history_repo.rs');
  const persistence = read('src-tauri/src/commands/persistence.rs');
  const settings = read('src-tauri/src/commands/privacy_settings.rs');
  assert.match(history, /DELETE FROM history WHERE visited_at < \?1/);
  assert.match(history, /prune_expired\(conn, retention, now\)/);
  assert.match(persistence, /history_repo::clear_history_data\(&tx\)/);
  assert.match(history, /site_icons_repo::clear\(conn\)/);
  assert.match(history, /closed_tabs_repo::clear\(conn\)/);
  assert.doesNotMatch(persistence, /DELETE FROM (?:site|download|cache|closed_tabs)/i);
  assert.match(settings, /history_repo::prune_expired\(conn, retention, now\)/);
});

test('pop-ups accept only enabled web destinations and keep the PiP path separate', () => {
  const tabs = read('src-tauri/src/commands/tabs.rs');
  const windows = read('src-tauri/src/window/webview_manager.rs');
  const pip = read('src-tauri/src/media/pip_window.rs');
  assert.match(tabs, /enabled\s*&&\s*matches!\(url\.scheme\(\), "http" \| "https"\)/);
  assert.match(windows, /pip_window::new_window[\s\S]*open_popup_url/);
  assert.match(pip, /about:blank#veloce-pip-/);
});

test('SmartScreen is applied to current and future Windows WebViews and is unavailable elsewhere', () => {
  const platform = read('src-tauri/src/security/mod.rs');
  const windows = read('src-tauri/src/security/windows.rs');
  const settings = read('src/lib/components/settings/PrivacySecuritySettings.svelte');
  assert.match(platform, /#\[cfg\(not\(windows\)\)\][\s\S]*unavailable on this platform/);
  assert.match(windows, /SetIsReputationCheckingRequired\(enabled\)/);
  assert.match(settings, /smart_screen_supported/);
});
