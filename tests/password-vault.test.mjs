import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { generatePassword, GENERATED_LENGTH } from '../src/lib/services/vault/password-generator.js';
import { copySecret, resetClipboardTimer, CLEAR_AFTER_MS } from '../src/lib/services/vault/clipboard.js';
import { vaultError } from '../src/lib/services/vault/errors.js';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

// A deterministic random source: counts upward, so every character choice is predictable.
const counter = () => { let n = 0; return array => { array[0] = n++ * 2654435761 % 0x100000000; return array; }; };

test('a generated password has every kind of character, the requested length and no look-alikes', () => {
  for (let run = 0; run < 50; run++) {
    const password = generatePassword();
    assert.equal(password.length, GENERATED_LENGTH);
    assert.match(password, /[a-z]/);
    assert.match(password, /[A-Z]/);
    assert.match(password, /[0-9]/);
    assert.match(password, /[^a-zA-Z0-9]/);
    assert.doesNotMatch(password, /[0OlI1]/);
  }
  assert.notEqual(generatePassword(), generatePassword());
});

test('the length is bounded and the random source is used without bias', () => {
  assert.equal(generatePassword(2).length, 4, 'room for one of each kind');
  assert.equal(generatePassword(500).length, 128);
  assert.equal(generatePassword(24, counter()).length, 24);
  // A value in the biased tail is thrown away and the source asked again.
  let calls = 0;
  const biased = array => { array[0] = calls++ === 0 ? 0xffffffff : 5; return array; };
  assert.ok(generatePassword(8, biased).length === 8 && calls > 8);
});

test('copying a secret clears the clipboard after 30 seconds and restarts the wait on a new copy', async () => {
  resetClipboardTimer();
  const written = [];
  const timers = [];
  const cleared = [];
  const env = {
    clipboard: { writeText: async text => { written.push(text); } },
    setTimer: (fn, ms) => { timers.push({ fn, ms }); return timers.length; },
    clearTimer: id => cleared.push(id),
  };
  assert.equal(await copySecret('first', env), true);
  assert.equal(timers[0].ms, CLEAR_AFTER_MS);
  assert.equal(CLEAR_AFTER_MS, 30_000);
  assert.equal(await copySecret('second', env), true);
  assert.deepEqual(cleared, [1], 'the earlier clean-up is cancelled');
  timers[1].fn();
  await new Promise(resolve => setImmediate(resolve));
  assert.deepEqual(written, ['first', 'second', '']);
});

test('a clipboard that refuses is reported and nothing is scheduled', async () => {
  resetClipboardTimer();
  const errors = [];
  const timers = [];
  const env = {
    clipboard: { writeText: async () => { throw new Error('denied'); } },
    setTimer: (fn, ms) => timers.push({ fn, ms }),
    onError: error => errors.push(error.message),
  };
  assert.equal(await copySecret('secret', env), false);
  assert.deepEqual(errors, ['denied']);
  assert.equal(timers.length, 0);
  // Without a clipboard at all there is nothing to copy to.
  assert.equal(await copySecret('secret', { clipboard: null, setTimer: () => 0 }) || await copySecret('secret', {}), false);
});

test('backend error codes become stable messages and never echo unknown text', () => {
  assert.deepEqual(vaultError('vault.wrong_secret'), { key: 'passwords.error_wrong_secret', values: {}, code: 'vault.wrong_secret' });
  assert.deepEqual(vaultError(new Error('vault.throttled:12')).values, { seconds: 12 });
  assert.equal(vaultError('vault.locked').key, 'passwords.error_locked');
  assert.equal(vaultError('vault.private_window').key, 'passwords.private_unavailable');
  const unknown = vaultError('something with a secret: hunter2');
  assert.equal(unknown.key, 'passwords.error_generic');
  assert.equal(unknown.code, 'vault.failed');
  assert.ok(!JSON.stringify(unknown).includes('hunter2'));
});

test('every message the vault screens use exists in both catalogs', () => {
  const en = JSON.parse(read('src/lib/i18n/catalogs/en/passwords.json'));
  const pt = JSON.parse(read('src/lib/i18n/catalogs/pt-BR/passwords.json'));
  assert.deepEqual(Object.keys(pt), Object.keys(en));
  const files = [
    ...readdirSync(new URL('../src/lib/components/settings/passwords/', import.meta.url)).map(f => `src/lib/components/settings/passwords/${f}`),
    'src/lib/services/vault/errors.js',
  ];
  for (const file of files) {
    for (const [, key] of read(file).matchAll(/["'](passwords\.[a-z_]+)["']/g)) assert.ok(key in en, `${key} in ${file}`);
  }
  assert.match(read('src/lib/i18n/catalogs.ts'), /passwordsEn[\s\S]*passwordsPt/);
});

test('Settings has a Passwords category with search entries and the not-audited warning', () => {
  assert.match(read('src/lib/stores/settingsStore.ts'), /\| "passwords"/);
  assert.match(read('src/lib/stores/settingsStore.ts'), /"passwords",\r?\n\s*"shortcuts"/);
  const catalog = read('src/lib/components/settings/settings-catalog.ts');
  assert.match(catalog, /id: "passwords", icon: "lock", label: "Passwords"/);
  assert.match(catalog, /category: "passwords", targetId: "passwords-heading"/);
  assert.match(read('src/lib/components/settings/SettingsModal.svelte'), /=== "passwords"\}\s*<PasswordSettings/);
  const screen = read('src/lib/components/settings/passwords/PasswordSettings.svelte');
  assert.match(screen, /id="passwords-heading"/);
  assert.match(screen, /passwords\.warning_title/);
  assert.ok(!/\{#if[^}]*warning/.test(screen), 'the warning is not conditional');
});

test('the vault screens never keep secrets around after use', () => {
  assert.match(read('src/lib/components/settings/passwords/VaultSetup.svelte'), /password = "";\s*confirmation = "";/);
  assert.match(read('src/lib/components/settings/passwords/VaultUnlock.svelte'), /secret = "";\s*if \(!value\) return;/);
  assert.match(read('src/lib/components/settings/passwords/EntryEditor.svelte'), /password = "";\s*notes = "";/);
  // The list only ever holds summaries; passwords are fetched one at a time on request.
  const types = read('src/lib/types/vault.ts');
  assert.match(types, /interface VaultSummary \{[^}]*\}/);
  assert.ok(!/interface VaultSummary \{[^}]*password/.test(types));
  const store = read('src/lib/stores/vaultStore.ts');
  assert.ok(!/localStorage|sessionStorage|safeStorage/.test(store), 'nothing is persisted by the interface');
});

test('every vault command is registered end to end', () => {
  const commands = ['vault_status', 'vault_create', 'vault_unlock', 'vault_unlock_recovery', 'vault_lock', 'vault_list', 'vault_reveal', 'vault_save_entry', 'vault_delete_entry', 'vault_change_master_password', 'vault_set_auto_lock', 'vault_reset'];
  const manifest = read('src-tauri/src/commands/manifest.rs');
  const capabilities = read('src-tauri/capabilities/default.json');
  const lib = read('src-tauri/src/lib.rs');
  const bridge = read('src/lib/services/bridge/vault.ts');
  const source = read('src-tauri/src/commands/vault.rs');
  for (const command of commands) {
    assert.ok(manifest.includes(`"${command}"`), `${command} in the manifest`);
    assert.ok(capabilities.includes(`"allow-${command.replaceAll('_', '-')}"`), `${command} in the capabilities`);
    assert.match(lib, new RegExp(`\\b${command},`), `${command} in lib.rs`);
    assert.ok(bridge.includes(`"${command}"`), `${command} in the bridge`);
    assert.ok(source.includes(`pub async fn ${command}(`) || source.includes(`pub fn ${command}(`), `${command} implemented`);
  }
  // Every command refuses private windows first.
  const bodies = source.split('#[tauri::command]').slice(1);
  assert.equal(bodies.length, commands.length);
  for (const body of bodies) assert.match(body, /allowed\(&app\)\?;/);
  assert.match(lib, /manage\(vault::service::VaultService::default\(\)\)/);
});

test('the vault never travels in backups and is not copied into private windows', () => {
  for (const file of ['src-tauri/src/backup/model.rs', 'src-tauri/src/backup/validation.rs', 'src-tauri/src/database/backup_repo.rs']) {
    assert.ok(!/vault/i.test(read(file)), `${file} does not mention the vault`);
  }
  const privateSession = read('src-tauri/src/private_session.rs');
  const copied = privateSession.slice(privateSession.indexOf('const COPIED_TABLES'), privateSession.indexOf('];', privateSession.indexOf('const COPIED_TABLES')));
  assert.ok(!/vault/.test(copied));
});

test('nothing in the vault code logs or prints a secret', () => {
  for (const file of ['src-tauri/src/vault/crypto.rs', 'src-tauri/src/vault/ops.rs', 'src-tauri/src/vault/service.rs', 'src-tauri/src/commands/vault.rs']) {
    const code = read(file).split('#[cfg(test)]')[0];
    assert.ok(!/println!|eprintln!|dbg!/.test(code), `${file} prints nothing`);
    for (const [line] of code.matchAll(/error_log::[a-z_]+\([^;]*;/gs)) {
      assert.ok(!/password|recovery_key|master|secret_value|\bkey\b/i.test(line.replace(/"[^"]*"/g, '')), `${file} logs only fixed text: ${line.slice(0, 80)}`);
    }
  }
});
