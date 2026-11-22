import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { lockError } from '../src/lib/services/appLock/errors.js';
import { vaultError } from '../src/lib/services/vault/errors.js';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
// Open or quit a locked profile: they answer before start-up is ready.
const UNLOCK_COMMANDS = ['get_lock_state', 'app_unlock_password', 'app_unlock_vault_password', 'app_unlock_hello', 'request_exit', 'erase_locked_profile'];
// Managed from Settings, in a profile that is already open.
const SETTINGS_COMMANDS = ['app_lock_enable', 'app_lock_add_method', 'app_lock_remove_method', 'app_lock_disable'];

test('every app lock command is registered end to end', () => {
  const manifest = read('src-tauri/src/commands/manifest.rs');
  const capabilities = read('src-tauri/capabilities/default.json');
  const lib = read('src-tauri/src/lib.rs');
  const bridge = read('src/lib/services/bridge/appLock.ts');
  const source = read('src-tauri/src/commands/app_lock.rs');
  for (const command of [...UNLOCK_COMMANDS, ...SETTINGS_COMMANDS]) {
    assert.ok(manifest.includes(`"${command}"`), `${command} in the manifest`);
    assert.ok(capabilities.includes(`"allow-${command.replaceAll('_', '-')}"`), `${command} in the capabilities`);
    assert.match(lib, new RegExp(`\\b${command},`), `${command} in lib.rs`);
    assert.ok(bridge.includes(`"${command}"`), `${command} in the bridge`);
    assert.match(source, new RegExp(`pub (async )?fn ${command}\\(`), `${command} implemented`);
  }
  assert.equal(source.split('#[tauri::command]').length - 1, UNLOCK_COMMANDS.length + SETTINGS_COMMANDS.length);
});

test('while locked only the unlock commands answer, and each of them checks that the profile is locked', () => {
  const access = read('src-tauri/src/commands/access.rs');
  const answers = /pub fn answers_before_ready[\s\S]*?\r?\n}\r?\n/.exec(access)[0];
  for (const command of UNLOCK_COMMANDS) assert.ok(answers.includes(`"${command}"`), `${command} answers before ready`);
  for (const command of SETTINGS_COMMANDS) assert.ok(!answers.includes(`"${command}"`), `${command} must wait for start-up (it is not an unlock command)`);
  const runtime = read('src/lib/services/bridge/runtime.ts');
  for (const command of UNLOCK_COMMANDS) assert.ok(runtime.includes(`"${command}"`), `${command} never waits for start-up`);
  const source = read('src-tauri/src/commands/app_lock.rs');
  const bodies = source.split('#[tauri::command]').slice(1);
  for (const body of bodies) {
    const name = /fn (\w+)\(/.exec(body)[1];
    if (['app_unlock_password', 'app_unlock_vault_password', 'app_unlock_hello', 'request_exit', 'erase_locked_profile'].includes(name)) {
      assert.match(body, /require_locked\(&service\)\?;|unlock_with\(/, `${name} requires a locked profile`);
    }
    if (SETTINGS_COMMANDS.includes(name)) assert.match(body, /allowed\(&app\)\?;/, `${name} refuses private windows`);
  }
  // The secrets that come in are wiped and never reach the log.
  assert.match(source, /Zeroizing::new\(password\)/);
  assert.doesNotMatch(source.replace(/"[^"]*"/g, '""'), /error_log::report\([^;]*(password|secret)/, 'no secret is logged');
});

test('a locked profile shows nothing of itself: the rest of start-up waits for the unlock', () => {
  const lib = read('src-tauri/src/lib.rs');
  assert.match(lib, /if app_lock::ops::is_enabled\(&conn\)[^{]*\{\s*app\.state::<app_lock::AppLockService>\(\)\.set_locked\(true\);/);
  assert.match(lib, /if app\.state::<app_lock::AppLockService>\(\)\.is_locked\(\) \{[\s\S]*?\} else \{\s*session_start::start\(app\.handle\(\)\)\?;/);
  // The pieces that load pages live in one place, run only by start-up and by the unlock.
  const start = read('src-tauri/src/session_start.rs');
  for (const piece of ['spawn_content_webview(app)', 'restore_saved', 'startup_ready()', 'feeds::start']) assert.ok(start.includes(piece), piece);
  assert.doesNotMatch(lib, /spawn_content_webview\(/, 'lib.rs no longer starts the content view itself');
  assert.doesNotMatch(lib, /restore_saved\(/);
  const commands = read('src-tauri/src/commands/app_lock.rs');
  assert.match(commands, /crate::session_start::start\(app\)/);
  assert.match(commands, /if !service\.open\(\) \{\s*return Ok\(\(\)\);/, 'simultaneous right answers start the profile once');
});

test('closing, quitting and a second launch of a locked profile need none of the usual interface', () => {
  const lib = read('src-tauri/src/lib.rs');
  assert.match(lib, /if app_lock::is_locked\(window\.app_handle\(\)\) \{\s*api\.prevent_close\(\);\s*app_lock::exit_locked\(window\.app_handle\(\)\);/);
  assert.match(lib, /if app_lock::is_locked\(app\) \{\s*app_lock::exit_locked\(app\);/);
  assert.match(read('src-tauri/src/instance_link.rs'), /if crate::app_lock::is_locked\(app\) \{[\s\S]*?set_focus\(\)[\s\S]*?return;/, 'a second launch only brings the lock screen forward');
  const mod = read('src-tauri/src/app_lock/mod.rs');
  assert.match(mod, /RunCheckpoint::CleanExit/, 'a locked exit still marks the run as ended normally');
});

test('the lock lives outside backups and private copies, and its secrets are only hashes', () => {
  const privateSession = read('src-tauri/src/private_session.rs');
  const copied = /const COPIED_TABLES[\s\S]*?\];/.exec(privateSession)[0];
  const vaultTables = /const VAULT_TABLES[\s\S]*?\];/.exec(privateSession)[0];
  assert.doesNotMatch(copied + vaultTables, /app_lock/, 'a private window never carries the lock');
  for (const file of ['backup/mod.rs', 'backup/model.rs']) {
    try { assert.doesNotMatch(read(`src-tauri/src/${file}`), /app_lock/, `${file} does not carry the lock`); } catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  const repo = read('src-tauri/src/database/app_lock_repo.rs');
  assert.match(repo, /not part of `BackupData`/);
  const verifier = read('src-tauri/src/app_lock/verifier.rs');
  assert.match(verifier, /Sha256::new\(\)/);
  assert.match(verifier, /same\(&self\.hash/, 'compared in constant time');
  assert.match(verifier, /crypto::derive_secret/, 'the password is stretched with the vault Argon2id');
  assert.match(read('src-tauri/src/database/connection.rs'), /app_lock_repo::migrate\(conn\)\?;/);
});

test('the brake is shared with the vault and survives a restart; the lock has its own Windows Hello key', () => {
  assert.match(read('src-tauri/src/vault/service.rs'), /crate::brake::delay_after/);
  assert.match(read('src-tauri/src/app_lock/ops.rs'), /crate::brake::delay_after/);
  assert.match(read('src-tauri/src/database/app_lock_repo.rs'), /blocked_until INTEGER NOT NULL/);
  const hello = read('src-tauri/src/vault/hello/mod.rs');
  assert.match(hello, /"SparkBrowserAppLock"/);
  assert.match(hello, /pub fn secret_app_lock/);
  assert.notEqual(/"SparkBrowserAppLock"/.test(hello), false);
});

test('the vault cannot be changed in a way that would lock the person out of the app', () => {
  const vault = read('src-tauri/src/vault/ops.rs');
  assert.match(vault, /pub fn matches_master_password/);
  assert.match(vault, /refuse_if_app_lock_needs_the_password\(conn\)\?;[\s\S]*?crypto::remove_password/);
  assert.match(vault, /pub fn reset[\s\S]*?refuse_if_app_lock_needs_the_password\(conn\)\?;/);
  assert.equal(vaultError('vault.required_by_app_lock').key, 'passwords.error_required_by_app_lock');
});

test('forgetting the secret erases the data of the profile and keeps the others', () => {
  const erase = read('src-tauri/src/app_lock/erase.rs');
  for (const kept of ['"profiles"', 'launcher::LAUNCHER_DIR', '"launcher.json"', '"profile.lock"']) assert.ok(erase.includes(kept), `${kept} is never erased from the Default profile`);
  const lib = read('src-tauri/src/lib.rs');
  assert.match(lib, /app_lock::erase::requested\(\)/);
  assert.ok(lib.indexOf('app_lock::erase::requested()') < lib.indexOf('profile::launcher::startup'), 'the helper is decided before any profile is chosen');
  assert.match(read('src-tauri/src/commands/app_lock.rs'), /if !confirm \{\s*return Err\("lock\.not_confirmed"\.into\(\)\);/);
});

test('the interface shows the lock screen before any of the browser, and Settings has the card', () => {
  const gate = read('src/LaunchGate.svelte');
  assert.match(gate, /else if \(launch\.locked\) mode = "locked"/);
  assert.match(gate, /<AppLockScreen onunlocked=\{showBrowser\} \/>/);
  assert.doesNotMatch(gate.replace(/import\("\.\/App\.svelte"\)/, ''), /App\.svelte/, 'the browser is not imported statically');
  assert.match(read('src/lib/components/settings/ProfilesSettings.svelte'), /<AppLockCard \/>/);
  assert.match(read('src/lib/components/settings/settings-catalog.ts'), /id: "app-lock", title: "App lock"/);
  const screen = read('src/lib/components/lock/AppLockScreen.svelte');
  assert.match(screen, /data-testid="app-lock-screen"/);
  assert.match(screen, /\{\$t\("lock\.not_encrypted"\)\}/, 'the screen says the lock does not encrypt');
  assert.match(read('src/lib/components/settings/AppLockCard.svelte'), /\$t\("lock\.settings_description"\)/);
});

test('every message the lock screens can show exists in both languages', () => {
  const used = new Set();
  for (const file of ['src/lib/components/lock/AppLockScreen.svelte', 'src/lib/components/settings/AppLockCard.svelte']) {
    for (const match of read(file).matchAll(/["'`](lock\.[a-z_]+)["'`]/g)) used.add(match[1]);
  }
  // In the error table the left side is a backend code; only the messages on the right are keys.
  for (const match of read('src/lib/services/appLock/errors.js').matchAll(/'(lock\.error_[a-z_]+)'/g)) used.add(match[1]);
  for (const way of ['on', 'off', 'changed']) used.add(`lock.notice_${way}`);
  assert.ok(used.size > 30);
  for (const language of ['en', 'pt-BR']) {
    const catalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/lock.json`));
    for (const key of used) assert.ok(key in catalog, `${key} in ${language}`);
  }
});

test('backend codes map to stable messages, with the seconds to wait', () => {
  assert.deepEqual(lockError('app_unlock_password: lock.wrong_secret'), { key: 'lock.error_wrong_secret', values: {}, code: 'lock.wrong_secret', seconds: 0 });
  const waiting = lockError(new Error('app_unlock_password: lock.wrong_secret_wait:7'));
  assert.equal(waiting.key, 'lock.error_wrong_secret_wait');
  assert.deepEqual(waiting.values, { seconds: 7 });
  assert.equal(lockError('lock.throttled:42').seconds, 42);
  assert.equal(lockError('vault.hello_cancelled').key, 'lock.error_hello_cancelled');
  assert.equal(lockError('lock.last_method').key, 'lock.error_last_method');
  assert.equal(lockError('something else').key, 'lock.error_failed');
  assert.equal(lockError(undefined).code, 'lock.failed');
});
