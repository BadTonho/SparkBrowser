import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(path, 'utf8');

test('automatic updates are part of the update state, on by default in the preview state', () => {
  assert.match(read('src/lib/types/updates.ts'), /auto_update: boolean;[\s\S]*staged: boolean;/);
  const bridge = read('src/lib/services/bridge/updates.ts');
  assert.match(bridge, /status: "unavailable", auto_update: true, staged: false/);
  assert.match(bridge, /invoke<UpdateState>\("set_auto_update", \{ enabled \}\)/);
});

test('the restart button skips the browser confirmation only when automatic updates are on', () => {
  const store = read('src/lib/stores/updatesStore.ts');
  assert.match(store, /export const setAutoUpdateEnabled = \(enabled: boolean\) => action\(async \(\) => \{ apply\(await changeAutoUpdate\(enabled\)\); \}\);/);
  assert.match(store, /if \(!state\.auto_update && !window\.confirm\(/);
  assert.match(store, /state\.status !== "ready"/, 'only a downloaded update can be installed');
});

test('settings show the toggle and the restart button, and the banner offers the restart once it is ready', () => {
  const settings = read('src/lib/components/settings/UpdateSettings.svelte');
  assert.match(settings, /id="setting-auto-update"[\s\S]*?id="auto-update" type="checkbox" checked=\{\$updateState\.auto_update\}/);
  assert.match(settings, /setAutoUpdateEnabled\(event\.currentTarget\.checked\)/);
  assert.match(settings, /\$updateState\.auto_update \? \$t\("settings\.restart_to_update"\) : \$t\("settings\.install_and_restart_1b99"\)/);
  assert.match(settings, /\$updateState\.auto_update && \$updateState\.staged \? \$t\("settings\.update_ready_next_start"\)/);
  const notice = read('src/lib/components/status/UpdateNotice.svelte');
  assert.match(notice, /\$updateState\.status === "ready" && \$updateState\.auto_update[\s\S]*?startUpdateInstall\(\)[\s\S]*?status\.restart_to_update/);
  assert.match(read('src/lib/components/settings/settings-catalog.ts'), /id: "auto-update"[^\n]*targetId: "setting-auto-update"/);
  for (const [language, label, restart] of [['en', 'Update automatically', 'Restart to update'], ['pt-BR', 'Atualizar automaticamente', 'Reiniciar para atualizar']]) {
    const settingsCatalog = JSON.parse(read(`src/lib/i18n/catalogs/${language}/settings.json`));
    assert.equal(settingsCatalog['settings.auto_update'], label);
    assert.equal(settingsCatalog['settings.restart_to_update'], restart);
    assert.equal(JSON.parse(read(`src/lib/i18n/catalogs/${language}/status.json`))['status.restart_to_update'], restart);
    for (const key of ['settings.auto_update_description', 'settings.update_ready_next_start', 'settings.update_downloading_automatically', 'settings.auto_update_search']) assert.ok(settingsCatalog[key], `${language} ${key}`);
  }
});

test('the backend downloads in the background, keeps the verified installer for the next start and installs it before restoring the session', () => {
  const commands = read('src-tauri/src/commands/updates.rs');
  assert.match(commands, /snapshot\.supported && snapshot\.auto_update && snapshot\.status == "available"/);
  assert.match(commands, /download_in_background\(&app, &snapshot\);\s*Ok\(snapshot\)/, 'a check starts the download');
  assert.match(commands, /pub fn set_auto_update/);
  assert.match(read('src-tauri/src/commands/manifest.rs'), /"set_auto_update"/);
  assert.match(read('src-tauri/src/lib.rs'), /\bset_auto_update,/);
  assert.match(read('src-tauri/capabilities/default.json'), /"allow-set-auto-update"/);
  const download = read('src-tauri/src/updates/windows.rs');
  assert.ok(download.indexOf('super::staged::write') > download.indexOf('candidate.download('), 'staged only after a verified download');
  assert.match(download, /if inner\.snapshot\.auto_update \{\s*to_stage = Some/, 'only with automatic updates on');
  assert.ok(download.indexOf('super::staged::clear_for(&app)') < download.indexOf('.install(bytes.as_slice())'), 'the restart button does not leave a second copy');
  const staged = read('src-tauri/src/updates/staged.rs');
  const install = staged.slice(staged.indexOf('pub fn install_pending'));
  assert.ok(install.indexOf('clear(&dir);') < install.indexOf('update.install('), 'one attempt only: files are removed before the installer runs');
  assert.ok(install.indexOf('update.verify(&installer)') < install.indexOf('update.install('), 'the signature is checked again at start');
  const lib = read('src-tauri/src/lib.rs');
  assert.ok(lib.indexOf('updates::install_staged_at_startup(app.handle(), &conn)') < lib.indexOf('recovery::startup::restore(&state, conn)'), 'before the run marker is touched');
  assert.match(read('src-tauri/src/updates/mod.rs'), /is_private\(\)/, 'private sessions never install');
});
