import assert from 'node:assert/strict';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { DatabaseSync } from 'node:sqlite';

/**
 * Separate profiles in the real app: they are created, renamed and deleted from Settings and from the
 * backend, the Default profile is protected, and opening a profile starts a second process with its
 * own folders (its own database and lock), which cannot be deleted while it runs.
 */
export async function checkProfiles(context) {
  const { invoke, until, script, click, withWebview, directory, request, sessionId } = context;
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
  await settings(() => click('[data-testid="settings-appearance_layout"]'));
  await settings(() => click('#setting-theme-mode button:first-of-type'));
  await settings(() => click('[data-testid="settings-profiles"]'));
  await until(() => settings(() => script('return document.querySelectorAll("[data-testid=profile-row]").length === 2')), 'two profile rows');
  const type = (selector, value) => settings(() => script(`const field=document.querySelector(${JSON.stringify(selector)});field.value=${JSON.stringify(value)};field.dispatchEvent(new Event('input',{bubbles:true}));return true`));
  const names = () => settings(() => script('return [...document.querySelectorAll("[data-testid=profile-name]")].map(item => item.textContent.trim())'));
  assert.equal((await names())[1], 'Work');
  assert.equal(await settings(() => script('return document.querySelectorAll("[data-testid=profile-current]").length')), 1, 'one profile is marked as in use');
  assert.equal(await settings(() => script('return !!document.querySelector("[data-testid=profiles-updates-note]")')), false, 'the Default profile installs updates, so there is no note');

  const create = '[data-testid="profile-create"]';
  const submit = '[data-testid="profile-create-submit"]';
  const picker = '[data-testid="profile-photo-picker"]';
  const togglePassword = '[data-testid="profile-password-toggle"]';
  const inputActions = (...actions) => settings(() => request('POST', `/session/${sessionId()}/actions`, { actions }));
  const dialogOpen = () => settings(() => script('return !!document.querySelector("[data-testid=profile-create-dialog][open]")'));
  await settings(() => click(create));
  await until(dialogOpen, 'the create-profile dialog opens');
  await until(() => settings(() => script('return document.activeElement?.id === "profile-create-name"')), 'the name field receives focus');
  await settings(() => script('document.activeElement.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true}));return true'));
  await until(() => settings(() => script('return !document.querySelector("[data-testid=profile-create-dialog]") && document.activeElement?.dataset.testid === "profile-create"')), 'Escape closes the dialog and restores focus');

  await settings(() => click(create));
  await until(dialogOpen, 'the create-profile dialog reopens');
  await until(() => settings(() => script('return getComputedStyle(document.querySelector("dialog")).colorScheme === "dark"')), 'the profile dialog follows the dark theme');
  const photoPickerPosition = await settings(() => script(`
    const avatar = document.querySelector(arguments[0]).getBoundingClientRect();
    const dialog = document.querySelector('[data-testid=profile-create-dialog]').getBoundingClientRect();
    return { x: Math.round(avatar.x + avatar.width / 2), y: Math.round(avatar.y + avatar.height / 2),
      offset: Math.abs(avatar.x + avatar.width / 2 - dialog.x - dialog.width / 2) };`, picker));
  assert.ok(photoPickerPosition.offset <= 1, 'the avatar is centered');
  const brushVisible = () => settings(() => script('return getComputedStyle(document.querySelector(".avatar-brush")).opacity === "1"'));
  await inputActions(
    { type: 'pointer', id: 'profile-hover', parameters: { pointerType: 'mouse' }, actions: [
      { type: 'pointerMove', duration: 0, x: photoPickerPosition.x, y: photoPickerPosition.y },
    ] },
  );
  await until(brushVisible, 'the avatar shows its brush on hover');
  // Observe the file-control activation without opening an operating-system dialog in WebDriver.
  await settings(() => script('const field = document.querySelector("[data-testid=profile-photo-input]"); field.click = () => { field.dataset.opens = String(Number(field.dataset.opens ?? 0) + 1); }; return true;'));
  await settings(() => click(picker));
  await settings(() => script('document.querySelector("#profile-create-name").focus();return true'));
  await inputActions(
    { type: 'pointer', id: 'profile-hover', parameters: { pointerType: 'mouse' }, actions: [{ type: 'pointerMove', duration: 0, x: 4, y: 4 }] },
    { type: 'key', id: 'profile-keyboard', actions: [
      { type: 'keyDown', value: '\uE008' }, { type: 'keyDown', value: '\uE004' },
      { type: 'keyUp', value: '\uE004' }, { type: 'keyUp', value: '\uE008' },
    ] },
  );
  assert.equal(await settings(() => script('return document.activeElement.matches(arguments[0] + ":focus-visible")', picker)), true);
  assert.equal(await brushVisible(), true, 'the brush stays visible with keyboard focus');
  await inputActions(
    { type: 'key', id: 'profile-keyboard', actions: [{ type: 'keyDown', value: '\uE007' }, { type: 'keyUp', value: '\uE007' }] },
  );
  assert.equal(await settings(() => script('return document.querySelector("[data-testid=profile-photo-input]").dataset.opens')), '2', 'both pointer and keyboard activate the file control');
  await type('[data-testid="profile-new-input"]', 'Home');
  await type('[data-testid="profile-new-password"]', 'short');
  const passwordState = () => settings(() => script(`
    const input = document.querySelector('[data-testid=profile-new-password]');
    const button = document.querySelector(arguments[0]);
    const field = input.getBoundingClientRect(), eye = button.getBoundingClientRect();
    return { type: input.type, value: input.value, label: button.getAttribute('aria-label'),
      icon: button.querySelector('path').getAttribute('d'), inside: eye.right <= field.right && eye.left > field.left + field.width / 2 };`, togglePassword));
  const hiddenPassword = await passwordState();
  assert.equal(hiddenPassword.type, 'password');
  assert.ok(hiddenPassword.inside);
  await settings(() => click(togglePassword));
  const shownPassword = await passwordState();
  assert.equal(shownPassword.type, 'text');
  assert.equal(shownPassword.value, hiddenPassword.value);
  assert.notEqual(shownPassword.label, hiddenPassword.label);
  assert.notEqual(shownPassword.icon, hiddenPassword.icon);
  await settings(() => click(togglePassword));
  assert.deepEqual(await passwordState(), hiddenPassword);
  assert.equal(await dialogOpen(), true);
  await settings(() => click(submit));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=profile-create-error]")')), 'a short password is explained');
  assert.deepEqual((await invoke('list_profiles')).items.map(item => item.name), ['', 'Work']);

  const choosePhoto = color => settings(() => script(`
    const canvas = document.createElement('canvas');
    canvas.width = 128; canvas.height = 96;
    const context = canvas.getContext('2d');
    context.fillStyle = arguments[0] || 'coral'; context.fillRect(0, 0, 128, 96);
    const bytes = arguments[0] ? Uint8Array.from(atob(canvas.toDataURL().split(',')[1]), char => char.charCodeAt(0)) : 'invalid';
    const transfer = new DataTransfer();
    transfer.items.add(new File([bytes], 'fixture.png', { type: 'image/png' }));
    const field = document.querySelector('[data-testid=profile-photo-input]');
    field.files = transfer.files;
    field.dispatchEvent(new Event('change', { bubbles: true }));
    return true;`, color));
  await choosePhoto(false);
  await until(() => settings(() => script('return document.querySelectorAll("[data-testid=profile-create-dialog] [role=alert]").length === 2')), 'invalid photo is explained');
  const preview = () => settings(() => script('return document.querySelector("[data-testid=profile-create-dialog] .profile-avatar img")?.src ?? null'));
  await choosePhoto('royalblue');
  await until(preview, 'the photo is previewed');
  const firstPhoto = await preview();
  await settings(() => click(picker));
  await settings(() => script('document.querySelector("[data-testid=profile-photo-input]").dispatchEvent(new Event("cancel"));return true'));
  assert.equal(await preview(), firstPhoto, 'cancel keeps the current photo');
  await choosePhoto('coral');
  await until(async () => (await preview()) !== firstPhoto, 'the photo is replaced');
  await settings(() => click('[data-testid="profile-photo-remove"]'));
  assert.equal(await preview(), null, 'removing the photo restores the avatar initial');
  await choosePhoto('coral');
  await until(preview, 'a photo can be selected again');
  await type('[data-testid="profile-new-password"]', 'native fixture password');
  await settings(() => context.screenshot('profile-create-dialog'));
  await settings(() => click(submit));
  await until(() => settings(() => script('return document.querySelectorAll("[data-testid=profile-row]").length === 3')), 'the profile created in Settings appears');
  await until(async () => !await dialogOpen(), 'the saved dialog closes');
  assert.deepEqual((await invoke('list_profiles')).items.map(item => item.name), ['', 'Work', 'Home']);
  const home = (await invoke('list_profiles')).items.find(item => item.name === 'Home');
  assert.match(home.avatar, /^data:image\/png;base64,/);
  const protectedDb = new DatabaseSync(join(profilesRoot, home.id, 'data', 'spark_browser.db'), { readOnly: true });
  try {
    const config = protectedDb.prepare('SELECT config FROM app_lock WHERE id = 1').get().config;
    assert.ok(JSON.parse(config).password, 'the profile stores a password verifier before its first launch');
    assert.ok(!config.includes('native fixture password'));
  } finally {
    protectedDb.close();
  }

  await settings(() => click(create));
  await until(dialogOpen, 'the create-profile dialog opens for a duplicate');
  await type('[data-testid="profile-new-input"]', 'work');
  await settings(() => click(submit));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=profile-create-error]")')), 'a repeated name is explained');
  assert.equal(await settings(() => script('return document.querySelector("[data-testid=profile-new-input]").value')), 'work');
  await settings(() => click('[data-testid="profile-create-cancel"]'));
  await until(async () => !await dialogOpen(), 'cancel closes the profile form');
  await settings(() => context.screenshot('profiles-settings'));

  await settings(() => click('[data-testid="settings-appearance_layout"]'));
  await settings(() => click('[data-testid="settings-display-tab"]'));
  const settingsTab = action => withWebview('/index.html?surface=settings-tab', action);
  await until(() => settingsTab(() => script('return !!document.querySelector("[data-testid=settings-profiles]")')), 'Settings opens as a tab');
  await settingsTab(() => click('[data-testid="settings-appearance_layout"]'));
  await settingsTab(() => click('#setting-theme-mode button:last-of-type'));
  await settingsTab(() => click('[data-testid="settings-profiles"]'));
  await until(() => settingsTab(() => script('return !!document.querySelector("[data-testid=profile-create]:enabled")')), 'tab shows profile creation');
  await settingsTab(() => click(create));
  await until(() => settingsTab(() => script('return !!document.querySelector("[data-testid=profile-create-dialog][open]")')), 'creation opens in the Settings tab');
  await until(() => settingsTab(() => script('return getComputedStyle(document.querySelector("dialog")).colorScheme === "light"')), 'the profile dialog follows the light theme');
  await settingsTab(() => context.screenshot('profile-create-light'));
  await settingsTab(() => script('const field=document.querySelector("[data-testid=profile-new-input]");field.value="Spare";field.dispatchEvent(new Event("input",{bubbles:true}));return true'));
  await settingsTab(() => click(submit));
  await until(async () => (await invoke('list_profiles')).items.some(item => item.name === 'Spare'), 'empty password allows profile creation');
  await until(() => settingsTab(() => script('return !document.querySelector("[data-testid=profile-create-dialog]")')), 'the Settings tab closes the saved dialog');
  const spare = (await invoke('list_profiles')).items.find(item => item.name === 'Spare');
  assert.equal(spare.avatar, null);
  assert.ok(!existsSync(join(profilesRoot, spare.id, 'data', 'spark_browser.db')), 'an empty password creates no lock database');
  await invoke('delete_profile', { id: spare.id });
  await settingsTab(() => click('[data-testid="settings-appearance_layout"]'));
  await settingsTab(() => click('[data-testid="settings-display-popup"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=settings-profiles]")')), 'Settings returns to the panel');
  await settings(() => click('[data-testid="settings-profiles"]'));
  await until(() => settings(() => script('return document.querySelectorAll("[data-testid=profile-row]").length === 3')), 'the saved profiles reappear');

  // Rename Home, then delete it after typing its name.
  const row = id => `[data-profile-id="${id}"]`;
  assert.equal(await settings(() => script(`return !!document.querySelector(${JSON.stringify(`${row(home.id)} .profile-avatar img`)})`)), true);
  await settings(() => click(`${row(home.id)} [data-testid="profile-rename"]`));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=profile-rename-input]")')), 'rename field');
  await type('[data-testid="profile-rename-input"]', 'Family');
  await settings(() => click('[data-testid="profile-rename-save"]'));
  await until(async () => (await invoke('list_profiles')).items.some(item => item.id === home.id && item.name === 'Family'), 'rename saved');
  assert.equal((await invoke('list_profiles')).items.find(item => item.id === home.id).avatar, home.avatar);
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

  const credential = 'first launch fixture';
  const locked = await invoke('create_profile', { name: 'Locked', avatar: home.avatar, password: credential });
  await context.restart({
    profileId: locked.id,
    lockScreen: true,
    beforeStart: () => writeFileSync(join(directory, 'profile', 'launcher.json'), JSON.stringify({ ask_every_time: false, last_profile: locked.id })),
  });
  assert.deepEqual(await invoke('get_launch_state'), { mode: 'browser', locked: true });
  assert.equal((await invoke('get_diagnostics')).startup_ready, false);
  await script('const field=document.querySelector("[data-testid=app-lock-password]");field.value=arguments[0];field.dispatchEvent(new Event("input",{bubbles:true}));return true', credential);
  await click('[data-testid="app-lock-submit"]');
  await until(() => script('return !!document.querySelector(".browser-root")'), 'the new profile opens after unlocking');
  await until(async () => (await invoke('get_diagnostics')).startup_ready, 'the new profile is ready');
  const afterRestart = await invoke('list_profiles');
  assert.equal(afterRestart.current, locked.id);
  assert.equal(afterRestart.items.find(item => item.id === locked.id).avatar, home.avatar);
  const errorLog = readFileSync(join(directory, 'data', 'logs', 'spark-errors.log'), 'utf8');
  assert.match(errorLog, /the new profile's app lock could not be initialized/);
  assert.ok(!errorLog.includes('native fixture password') && !errorLog.includes(credential));
}
