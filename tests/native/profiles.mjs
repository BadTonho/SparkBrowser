import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

/**
 * Separate profiles in the real app: they are created, renamed and deleted from Settings and from the
 * backend, the Default profile is protected, and opening a profile starts a second process with its
 * own folders (its own database and lock), which cannot be deleted while it runs.
 */
export async function checkProfiles(context) {
  const { invoke, until, script, click, withWebview, directory } = context;
  const settings = action => withWebview('/index.html?surface=settings', action);
  const profilesRoot = join(directory, 'profile', 'profiles');
  const refusal = async (command, args) => {
    try { await invoke(command, args); return ''; } catch (error) { return String(error?.message ?? error); }
  };

  // Backend: the Default profile is always there, new ones are validated and get their own folder.
  const first = await invoke('list_profiles');
  assert.equal(first.current, 'default');
  assert.equal(first.items.length, 1);
  assert.equal(first.items[0].is_default, true);
  assert.equal(first.max, 10);
  const work = await invoke('create_profile', { name: '  Work  ' });
  assert.equal(work.name, 'Work');
  assert.match(work.id, /^[0-9a-f]{12}$/);
  assert.ok(existsSync(join(profilesRoot, work.id, 'profile.json')), 'the profile folder was written');
  assert.match(await refusal('create_profile', { name: 'work' }), /profile\.duplicate/);
  assert.match(await refusal('create_profile', { name: '   ' }), /profile\.invalid_name/);
  assert.match(await refusal('create_profile', { name: 'a'.repeat(41) }), /profile\.invalid_name/);
  assert.match(await refusal('rename_profile', { id: 'default', name: 'x' }), /profile\.cannot_rename_default/);
  assert.match(await refusal('delete_profile', { id: 'default' }), /profile\.cannot_delete_default/);
  assert.match(await refusal('delete_profile', { id: '../evil' }), /profile\.not_found/);
  assert.match(await refusal('delete_profile', { id: 'ffffffffffff' }), /profile\.not_found/);
  assert.match(await refusal('open_profile', { id: 'ffffffffffff' }), /profile\.not_found/);
  assert.deepEqual((await invoke('list_profiles')).items.map(item => item.name), ['', 'Work']);

  // Settings: the Profiles category lists them, creates, renames and deletes with a typed confirmation.
  await context.focusWindow();
  await click('[data-testid="open-settings"]');
  await until(() => withWebview('/index.html?surface=settings-quick', () => script('return !!document.querySelector("[data-testid=settings-quick-all]")')), 'quick settings');
  await withWebview('/index.html?surface=settings-quick', () => click('[data-testid="settings-quick-all"]'));
  await until(() => settings(() => script('return !!document.querySelector("#settings-search-input")')), 'settings dialog');
  await settings(() => click('[data-testid="settings-profiles"]'));
  await until(() => settings(() => script('return document.querySelectorAll("[data-testid=profile-row]").length === 2')), 'two profile rows');
  const type = (selector, value) => settings(() => script(`const field=document.querySelector(${JSON.stringify(selector)});field.value=${JSON.stringify(value)};field.dispatchEvent(new Event('input',{bubbles:true}));return true`));
  const names = () => settings(() => script('return [...document.querySelectorAll("[data-testid=profile-name]")].map(item => item.textContent.trim())'));
  assert.equal((await names())[1], 'Work');
  assert.equal(await settings(() => script('return document.querySelectorAll("[data-testid=profile-current]").length')), 1, 'one profile is marked as in use');
  assert.equal(await settings(() => script('return !!document.querySelector("[data-testid=profiles-updates-note]")')), false, 'the Default profile installs updates, so there is no note');

  await type('[data-testid="profile-new-input"]', 'Home');
  await settings(() => click('[data-testid="profile-create"]'));
  await until(() => settings(() => script('return document.querySelectorAll("[data-testid=profile-row]").length === 3')), 'the profile created in Settings appears');
  assert.deepEqual((await invoke('list_profiles')).items.map(item => item.name), ['', 'Work', 'Home']);
  await type('[data-testid="profile-new-input"]', 'work');
  await settings(() => click('[data-testid="profile-create"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=profiles-error]")')), 'a repeated name is explained');
  await settings(() => context.screenshot('profiles-settings'));

  // Rename Home, then delete it after typing its name.
  const home = (await invoke('list_profiles')).items.find(item => item.name === 'Home');
  const row = id => `[data-profile-id="${id}"]`;
  await settings(() => click(`${row(home.id)} [data-testid="profile-rename"]`));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=profile-rename-input]")')), 'rename field');
  await type('[data-testid="profile-rename-input"]', 'Family');
  await settings(() => click('[data-testid="profile-rename-save"]'));
  await until(async () => (await invoke('list_profiles')).items.some(item => item.id === home.id && item.name === 'Family'), 'rename saved');
  await settings(() => click(`${row(home.id)} [data-testid="profile-delete"]`));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=profile-delete-input]")')), 'delete confirmation');
  assert.equal(await settings(() => script('return document.querySelector("[data-testid=profile-delete-submit]").disabled')), true, 'deleting waits for the name');
  await type('[data-testid="profile-delete-input"]', 'Wrong');
  assert.equal(await settings(() => script('return document.querySelector("[data-testid=profile-delete-submit]").disabled')), true);
  await type('[data-testid="profile-delete-input"]', 'Family');
  await until(() => settings(() => script('return !document.querySelector("[data-testid=profile-delete-submit]").disabled')), 'the name was typed');
  await settings(() => click('[data-testid="profile-delete-submit"]'));
  await until(async () => (await invoke('list_profiles')).items.length === 2, 'the profile was deleted');
  assert.ok(!existsSync(join(profilesRoot, home.id)), 'its folder is gone');

  // Opening a profile starts another process with its own folders; it cannot be deleted while it runs.
  await invoke('open_profile', { id: work.id });
  const database = join(profilesRoot, work.id, 'data', 'spark_browser.db');
  await until(() => existsSync(database), 'the second process created its own database', 60_000);
  assert.match(await refusal('delete_profile', { id: work.id }), /profile\.in_use/);
  // The window of that profile says which profile it is.
  if (process.platform === 'win32') {
    const titleOf = () => spawnSync('powershell', ['-NoProfile', '-Command', `Get-CimInstance Win32_Process | Where-Object { $_.Name -like 'spark-browser*' -and $_.CommandLine -like '*--profile ${work.id}*' } | ForEach-Object { (Get-Process -Id $_.ProcessId).MainWindowTitle }`], { encoding: 'utf8' }).stdout.trim();
    await until(() => titleOf() === 'Spark - Work', 'the window of the profile is titled with its name', 30_000);
  }
  assert.ok(existsSync(join(profilesRoot, work.id)), 'a running profile is not deleted');
  const stop = process.platform === 'win32'
    ? spawnSync('powershell', ['-NoProfile', '-Command', `Get-CimInstance Win32_Process | Where-Object { $_.Name -like 'spark-browser*' -and $_.CommandLine -like '*--profile ${work.id}*' } | ForEach-Object { taskkill /PID $_.ProcessId /T /F 2>&1 | Out-Null }; exit 0`], { encoding: 'utf8' })
    : spawnSync('pkill', ['-f', `--profile ${work.id}`], { encoding: 'utf8' });
  assert.equal(stop.status, 0, `the second process was stopped: status ${stop.status} ${stop.error ?? ""} ${stop.stdout} ${stop.stderr}`);
  await until(async () => (await refusal('delete_profile', { id: work.id })) === '', 'the profile can be deleted once it is closed', 30_000);
  assert.ok(!existsSync(join(profilesRoot, work.id)), 'the folder of the closed profile is gone');
  assert.deepEqual((await invoke('list_profiles')).items.map(item => item.id), ['default']);
}
