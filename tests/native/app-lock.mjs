import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { DatabaseSync } from 'node:sqlite';

/**
 * The app lock of a profile, in the real app. It is turned on from Settings; the next start shows only
 * the lock screen, with no tab, page or working command behind it; wrong secrets are slowed down; the
 * right one opens the browser; and "forgot the secret" erases the profile's data and starts it again
 * without a lock. Only artificial secrets are used.
 */
export async function checkAppLock(context) {
  const { invoke, until, script, click, withWebview, directory, restart, screenshot, wait, forgetSession } = context;
  const settings = action => withWebview('/index.html?surface=settings', action);
  const refusal = async (command, args) => {
    try { await invoke(command, args); return ''; } catch (error) { return String(error?.message ?? error); }
  };
  const first = 'correct horse battery';
  const second = 'another long password';

  // A profile starts without a lock. A short password is refused and leaves it off.
  let state = await invoke('get_lock_state');
  assert.equal(state.locked, false);
  assert.equal(state.enabled, false);
  assert.match(await refusal('app_lock_enable', { method: 'password', secret: 'short' }), /lock\.password_invalid/);
  assert.equal((await invoke('get_lock_state')).enabled, false);
  // Erase-me data that the lock screen's "forgot" must destroy.
  await invoke('create_bookmark_folder', { name: 'Erase me' });

  await invoke('app_lock_enable', { method: 'password', secret: first });
  state = await invoke('get_lock_state');
  assert.deepEqual([state.enabled, state.password, state.hello, state.vault], [true, true, false, false]);
  assert.match(await refusal('app_lock_enable', { method: 'password', secret: second }), /lock\.already_on/);
  assert.match(await refusal('app_lock_remove_method', { method: 'password', proof: { method: 'password', secret: first } }), /lock\.last_method/, 'the last way in cannot be removed');
  assert.match(await refusal('app_unlock_password', { password: first }), /lock\.not_locked/, 'an open profile has nothing to unlock');
  assert.match(await refusal('erase_locked_profile', { confirm: true }), /lock\.not_locked/, 'only a locked profile can be erased from the lock screen');

  // Settings: the card shows the lock and changes the password after asking for the current one.
  await context.focusWindow();
  await click('[data-testid="open-settings"]');
  await until(() => withWebview('/index.html?surface=settings-quick', () => script('return !!document.querySelector("[data-testid=settings-quick-all]")')), 'quick settings');
  await withWebview('/index.html?surface=settings-quick', () => click('[data-testid="settings-quick-all"]'));
  await until(() => settings(() => script('return !!document.querySelector("#settings-search-input")')), 'settings dialog');
  await settings(() => click('[data-testid="settings-profiles"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=app-lock-ways]")')), 'the app lock card shows the ways in');
  assert.equal(await settings(() => script('return document.querySelector("[data-testid=app-lock-way-password]").dataset.on')), 'true');
  assert.equal(await settings(() => script('return document.querySelector("[data-testid=app-lock-way-hello]").dataset.on')), 'false');
  const type = (selector, value) => settings(() => script(`const field=document.querySelector(${JSON.stringify(selector)});field.value=${JSON.stringify(value)};field.dispatchEvent(new Event('input',{bubbles:true}));return true`));
  await type('[data-testid="app-lock-change-input"]', second);
  await settings(() => click('[data-testid="app-lock-change-password-submit"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=app-lock-confirm-panel]")')), 'the card asks who is asking');
  await type('[data-testid="app-lock-confirm-secret"]', 'not the current one');
  await settings(() => click('[data-testid="app-lock-confirm-submit"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=app-lock-confirm-error]")')), 'a wrong current secret is refused');
  await type('[data-testid="app-lock-confirm-secret"]', first);
  await settings(() => click('[data-testid="app-lock-confirm-submit"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=app-lock-notice]")')), 'the change is confirmed');
  await settings(() => screenshot('app-lock-settings'));

  // The next start is the lock screen, and nothing else works behind it.
  await restart({ lockScreen: true });
  assert.deepEqual(await invoke('get_launch_state'), { mode: 'browser', locked: true });
  assert.equal((await invoke('get_diagnostics')).startup_ready, false, 'the profile does not start while locked');
  for (const command of ['get_tabs', 'list_profiles', 'get_history', 'vault_status']) assert.match(await refusal(command), /still in progress/i, `${command} is refused while locked`);
  assert.match(await refusal('app_lock_disable', { proof: { method: 'password', secret: second } }), /still in progress/i, 'the lock cannot be turned off from behind the lock');
  state = await invoke('get_lock_state');
  assert.deepEqual([state.locked, state.enabled, state.password], [true, true, true]);
  assert.equal(await script('return !!document.querySelector(".browser-root, .titlebar, .h-tabbar")'), false, 'none of the browser is on screen');
  await screenshot('app-lock-screen');

  // Wrong secrets: the first is refused in the interface, then the brake starts.
  await script('const f=document.querySelector("[data-testid=app-lock-password]");f.value="definitely wrong";f.dispatchEvent(new Event("input",{bubbles:true}));return true');
  await script('document.querySelector("[data-testid=app-lock-submit]").click();return true');
  await until(() => script('return !!document.querySelector("[data-testid=app-lock-error]")'), 'a wrong secret is explained on the lock screen');
  assert.match(await refusal('app_unlock_password', { password: first }), /lock\.wrong_secret/, 'the old password no longer opens it');
  const third = await refusal('app_unlock_password', { password: 'still wrong' });
  assert.match(third, /lock\.wrong_secret_wait:\d+/, 'the third wrong guess starts the wait');
  // Another guess right after is either held back by the wait or, if a second already passed, counts as one
  // more (the exact timing of the brake is covered by unit tests). A wrong secret never opens the profile.
  assert.match(await refusal('app_unlock_password', { password: 'wrong again' }), /lock\.(throttled|wrong_secret_wait):\d+/);
  assert.match(await refusal('app_unlock_vault_password', { password: 'wrong again' }), /lock\.(throttled|method_unavailable|wrong_secret)/);
  await wait(3500);

  // The right password opens the browser, which then works as usual.
  await script('const f=document.querySelector("[data-testid=app-lock-password]");f.value=arguments[0];f.dispatchEvent(new Event("input",{bubbles:true}));return true', second);
  await until(() => script('return !document.querySelector("[data-testid=app-lock-submit]").disabled'), 'the wait is over');
  await script('document.querySelector("[data-testid=app-lock-submit]").click();return true');
  await until(() => script('return !!document.querySelector(".browser-root")'), 'the browser shows itself after unlocking', 60_000);
  await until(async () => (await invoke('get_diagnostics')).startup_ready, 'the profile finished starting');
  assert.ok((await invoke('get_tabs')).tabs.length > 0, 'the session came back');
  assert.equal((await invoke('get_lock_state')).locked, false);
  assert.match(await refusal('app_unlock_password', { password: second }), /lock\.not_locked/);

  // Forgot the secret: the data of the profile is erased and the profile starts again, empty and unlocked.
  assert.ok((await invoke('get_bookmark_data')).folders.some(folder => folder.name === 'Erase me'), 'the data to erase is there');
  await restart({ lockScreen: true });
  assert.match(await refusal('erase_locked_profile', { confirm: false }), /lock\.not_confirmed/, 'erasing needs an explicit confirmation');
  await script('document.querySelector("[data-testid=app-lock-forgot]").click();return true');
  await until(() => script('return !!document.querySelector("[data-testid=app-lock-forgot-panel]")'), 'the warning is shown');
  await script('document.querySelector("[data-testid=app-lock-forgot-cancel]").click();return true');
  await until(() => script('return !!document.querySelector("[data-testid=app-lock-password]")'), 'cancelling goes back to the lock screen');
  await script('document.querySelector("[data-testid=app-lock-forgot]").click();return true');
  await until(() => script('return !!document.querySelector("[data-testid=app-lock-forgot-confirm]")'), 'the warning is shown again');
  await screenshot('app-lock-forgot');
  const databasePath = join(directory, 'data', 'spark_browser.db');
  await script('document.querySelector("[data-testid=app-lock-forgot-confirm]").click();return true');
  forgetSession();
  const stop = () => process.platform === 'win32' && spawnSync('powershell', ['-NoProfile', '-Command', "Get-CimInstance Win32_Process | Where-Object { $_.Name -like 'spark-browser*' -and $_.CommandLine -like '*--profile default*' } | ForEach-Object { taskkill /PID $_.ProcessId /T /F 2>&1 | Out-Null }; exit 0"], { encoding: 'utf8' });
  const started = () => spawnSync('powershell', ['-NoProfile', '-Command', "(Get-CimInstance Win32_Process | Where-Object { $_.Name -like 'spark-browser*' -and $_.CommandLine -like '*--profile default*' } | Measure-Object).Count"], { encoding: 'utf8' }).stdout.trim() !== '0';
  try {
    if (process.platform === 'win32') await until(() => started(), 'the helper started the profile again', 90_000);
    await until(() => existsSync(databasePath), 'the profile has a database again', 60_000);
    await wait(1500);
    const database = new DatabaseSync(databasePath, { readOnly: true });
    try {
      assert.equal(database.prepare("SELECT count(*) AS n FROM bookmark_folders WHERE name='Erase me'").get().n, 0, 'the old data is gone');
      assert.equal(database.prepare('SELECT count(*) AS n FROM app_lock').get().n, 0, 'the lock is gone with it');
    } finally { database.close(); }
  } finally { stop(); }
  await wait(1000);
}
