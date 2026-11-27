import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = (path) => readFileSync(path, 'utf8').replace(/\r\n/g, '\n');

test('the Nextcloud sign-in commands are registered end to end', () => {
  for (const command of ['nextcloud_login_start', 'nextcloud_login_poll', 'nextcloud_login_cancel']) {
    assert.match(read('src-tauri/src/commands/manifest.rs'), new RegExp(`"${command}"`));
    assert.match(read('src-tauri/capabilities/default.json'), new RegExp(`"allow-${command.replaceAll('_', '-')}"`));
    assert.match(read('src-tauri/src/lib.rs'), new RegExp(`${command},\\n`));
    assert.match(read('src/lib/services/bridge/backup.ts'), new RegExp(`"${command}"`));
  }
  assert.match(read('src-tauri/src/lib.rs'), /\.manage\(backup::nextcloud::NextcloudLogin::default\(\)\)/);
});

test('Nextcloud is offered as its own sync mode and every sign-in error has a message in both languages', () => {
  const page = read('src/lib/components/settings/BackupSettings.svelte');
  assert.match(page, /<option value="nextcloud">/);
  const errors = read('src/lib/i18n/errors.ts');
  const catalogs = ['en', 'pt-BR'].map((lang) => JSON.parse(read(`src/lib/i18n/catalogs/${lang}/backup.json`)));
  for (const code of ['invalid_server', 'not_found', 'unexpected_server', 'login_cancelled', 'login_expired']) {
    assert.match(errors, new RegExp(`"backup.nextcloud_${code}": "backup.error_nextcloud_${code}"`));
    for (const catalog of catalogs) assert.ok(catalog[`backup.error_nextcloud_${code}`], code);
  }
  for (const key of ['sync_mode_nextcloud', 'sync_title_nextcloud', 'nextcloud_sign_in', 'nextcloud_waiting', 'nextcloud_connected', 'nextcloud_hint']) {
    for (const catalog of catalogs) assert.ok(catalog[`backup.${key}`], key);
  }
});

test('the app password never reaches the interface', () => {
  const bridge = read('src/lib/services/bridge/backup.ts');
  const store = read('src/lib/stores/nextcloudLoginStore.ts');
  for (const source of [bridge, store]) assert.doesNotMatch(source, /appPassword|app_password/);
  // A Nextcloud account is sent and kept like the other targets: a WebDAV folder whose secret stays in Rust.
  assert.match(read('src-tauri/src/commands/backup_nextcloud.rs'), /Some\(account\.app_password\)/);
});
