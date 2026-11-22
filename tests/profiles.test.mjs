import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { profileError } from '../src/lib/services/profiles/errors.js';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const COMMANDS = ['list_profiles', 'create_profile', 'rename_profile', 'delete_profile', 'open_profile'];

test('every profile command is registered end to end and refuses private windows', () => {
  const manifest = read('src-tauri/src/commands/manifest.rs');
  const capabilities = read('src-tauri/capabilities/default.json');
  const lib = read('src-tauri/src/lib.rs');
  const bridge = read('src/lib/services/bridge/profiles.ts');
  const source = read('src-tauri/src/commands/profiles.rs');
  for (const command of COMMANDS) {
    assert.ok(manifest.includes(`"${command}"`), `${command} in the manifest`);
    assert.ok(capabilities.includes(`"allow-${command.replaceAll('_', '-')}"`), `${command} in the capabilities`);
    assert.match(lib, new RegExp(`\\b${command},`), `${command} in lib.rs`);
    assert.ok(bridge.includes(`"${command}"`), `${command} in the bridge`);
    assert.ok(source.includes(`pub async fn ${command}(`), `${command} implemented`);
  }
  const bodies = source.split('#[tauri::command]').slice(1);
  assert.equal(bodies.length, COMMANDS.length);
  for (const body of bodies) assert.match(body, /allowed\(&app\)\?;/, 'private windows are refused first');
});

test('a profile id is validated before it reaches a path, and a profile process redirects every directory', () => {
  const store = read('src-tauri/src/profiles/store.rs');
  assert.match(store, /fn valid_id[\s\S]*len\(\) == 12/);
  const mod = read('src-tauri/src/profiles/mod.rs');
  for (const directory of ['config', 'data', 'local_data', 'cache', 'log']) assert.match(mod, new RegExp(`${directory}: Some\\(folder`), directory);
  const lib = read('src-tauri/src/lib.rs');
  assert.match(lib, /profiles::configure\(&mut context, profile_id\.as_deref\(\), &local_root\)/);
  // Private windows are not profiles, and the legacy folder migration is only for the Default profile.
  assert.match(lib, /if role\.is_private\(\) \{\s*None\s*\}/);
  assert.match(lib, /!role\.is_private\(\) && profile_id\.is_none\(\)/);
});

test('profiles never share a Windows Hello key, and only the Default profile installs updates', () => {
  const hello = read('src-tauri/src/vault/hello/mod.rs');
  assert.match(hello, /pub fn use_profile/);
  assert.match(hello, /format!\("\{base\}-\{id\}"\)/);
  const windows = read('src-tauri/src/vault/hello/platform_windows.rs');
  assert.doesNotMatch(windows, /HSTRING::from\(CREDENTIAL\)/, 'the key name always passes through the profile');
  assert.match(read('src-tauri/src/updates/mod.rs'), /crate::profiles::is_default_profile\(app\)/);
  assert.match(read('src-tauri/src/lib.rs'), /vault::hello::use_profile\(profile_id\.as_deref\(\)\)/);
});

test('the Profiles category is reachable from Settings and searchable', () => {
  assert.match(read('src/lib/components/settings/settings-catalog.ts'), /id: "profiles", icon: "user", label: "Profiles"/);
  assert.match(read('src/lib/components/settings/settings-catalog.ts'), /category: "profiles", targetId: "profiles-heading"/);
  assert.match(read('src/lib/stores/settingsStore.ts'), /\| "profiles"/);
  assert.match(read('src/lib/components/settings/SettingsModal.svelte'), /=== "profiles"\}\s*<ProfilesSettings \/>/);
});

test('every message the profiles screen can show exists in both languages', () => {
  const component = read('src/lib/components/settings/ProfilesSettings.svelte') + read('src/lib/services/profiles/errors.js') + read('src/lib/stores/profilesStore.ts');
  const used = new Set([...component.matchAll(/["'`](profiles\.[a-z_]+)["'`]/g)].map(match => match[1]));
  assert.ok(used.size > 15);
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/profiles.json`));
    for (const key of used) assert.ok(key in catalog, `${key} in ${language}`);
  }
  assert.ok(readdirSync(new URL('../src/lib/i18n/catalogs/en', import.meta.url)).includes('profiles.json'));
});

test('backend error codes map to stable messages and unknown ones to the generic one', () => {
  assert.equal(profileError('profile.duplicate').key, 'profiles.error_duplicate');
  assert.equal(profileError(new Error('open_profile: profile.in_use')).key, 'profiles.error_in_use');
  assert.equal(profileError('profile.delete_failed').key, 'profiles.error_failed');
  assert.equal(profileError('something else').key, 'profiles.error_failed');
  assert.equal(profileError(undefined).code, 'profile.failed');
});

test('a profile other than the Default one explains on the Updates page why it does not update', () => {
  const page = read('src/lib/components/settings/UpdateSettings.svelte');
  assert.match(page, /const otherProfile = \$derived\(!!\$profiles && \$profiles\.current !== "default"\)/);
  assert.match(page, /\{#if otherProfile\}<p role="note" data-testid="updates-other-profile">\{\$t\("profiles\.updates_note"\)\}<\/p>\{\/if\}/);
});
