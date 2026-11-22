import assert from 'node:assert/strict';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { validateRunDirectory } from './run-directory.mjs';

/**
 * The profile picker at start-up, in the real app. With two profiles and "ask every time" on, Spark
 * starts as a picker with no profile of its own: it answers only the picker commands, loads no browser,
 * and choosing a profile starts that profile in its own process, remembers the choice and ends the
 * picker. The next start then opens the remembered profile straight away.
 */
export async function checkProfilePicker(context) {
  const { invoke, until, script, directory, restart, screenshot, wait, forgetSession, resizeWindow, request, sessionId } = context;
  const root = validateRunDirectory(directory);
  const profilesRoot = join(root, 'profile', 'profiles');
  const launcherFile = join(root, 'profile', 'launcher.json');
  const refusal = async (command, args) => {
    try { await invoke(command, args); return ''; } catch (error) { return String(error?.message ?? error); }
  };
  const killProfile = id => process.platform === 'win32' && spawnSync('powershell', ['-NoProfile', '-Command', `Get-CimInstance Win32_Process | Where-Object { $_.Name -like 'spark-browser*' -and $_.CommandLine -like '*--profile ${id}*' } | ForEach-Object { taskkill /PID $_.ProcessId /T /F 2>&1 | Out-Null }; exit 0`], { encoding: 'utf8' });
  const visibleCards = () => script('return document.querySelectorAll("[data-testid=profile-picker-item]").length');
  async function checkRows(expected) {
    const rows = await script(`
      const list = document.querySelector('[data-testid=profile-picker-list]');
      const bounds = list.getBoundingClientRect();
      const center = bounds.left + bounds.width / 2;
      const rows = new Map();
      for (const card of list.querySelectorAll('[data-testid=profile-picker-item]')) {
        const rect = card.getBoundingClientRect();
        const top = Math.round(rect.top);
        if (!rows.has(top)) rows.set(top, []);
        rows.get(top).push(rect);
      }
      return [...rows.values()].map(cards => ({ count: cards.length,
        centered: Math.abs((cards[0].left + cards.at(-1).right) / 2 - center) < 2,
        vertical: cards.every(card => card.height >= card.width) }));`);
    assert.deepEqual(rows, expected.map(count => ({ count, centered: true, vertical: true })));
  }
  const press = value => request('POST', `/session/${sessionId()}/actions`, { actions: [
    { type: 'key', id: 'profile-picker-keyboard', actions: [{ type: 'keyDown', value }, { type: 'keyUp', value }] },
  ] });

  // With a single profile there is nothing to choose; the toggle is saved for the next start.
  assert.deepEqual(await invoke('get_launch_state'), { mode: 'browser', locked: false });
  assert.deepEqual(await invoke('get_launcher_prefs'), { ask_every_time: false });
  const avatar = await script(`const canvas = document.createElement('canvas');
    canvas.width = canvas.height = 96;
    const context = canvas.getContext('2d');
    context.fillStyle = 'coral'; context.fillRect(0, 0, 96, 96);
    return canvas.toDataURL('image/png');`);
  const work = await invoke('create_profile', { name: 'Work', avatar });
  await invoke('set_ask_profile_at_startup', { enabled: true });
  assert.deepEqual(await invoke('get_launcher_prefs'), { ask_every_time: true });
  assert.match(await refusal('get_picker_state'), /profile\.not_picker/, 'only the picker answers the picker commands');
  assert.match(await refusal('choose_profile', { id: work.id, askEveryTime: false }), /profile\.not_picker/);

  // The next start is the picker.
  await restart({ launcher: true });
  assert.deepEqual(await invoke('get_launch_state'), { mode: 'picker', locked: false });
  assert.equal((await invoke('get_diagnostics')).startup_ready, false, 'the picker never becomes the browser');
  assert.match(await refusal('get_tabs'), /start-up is still in progress|startup is still in progress/i, 'no browser command works in the picker');
  assert.match(await refusal('list_profiles'), /still in progress/i);
  const picker = await invoke('get_picker_state');
  assert.deepEqual(picker.profiles.map(profile => profile.name), ['', 'Work']);
  assert.equal(picker.profiles[1].avatar, work.avatar);
  assert.equal(picker.ask_every_time, true);
  assert.equal(picker.last_profile, null, 'nothing was chosen yet');
  await until(async () => await visibleCards() === 2, 'the two profile cards appear');
  assert.equal(await script(`return document.querySelector('[data-profile-id="${work.id}"] img').getAttribute('src')`), work.avatar);
  assert.equal(await script(`return document.querySelector('[data-profile-id="default"] .profile-avatar').textContent.trim().length`), 1);
  assert.equal(await script('return document.querySelector("[data-testid=profile-picker-ask]").checked'), true);
  assert.equal(await script('return !!document.querySelector(".browser-root, .titlebar, .h-tabbar")'), false, 'none of the browser is on screen');
  await checkRows([2]);
  await screenshot('profile-picker');
  await resizeWindow(680, 480);
  await until(() => script('return innerWidth < 704'), 'picker is at minimum width');
  await checkRows([2]);
  await screenshot('profile-picker-minimum');

  // Add fixture profiles to the isolated store while the picker is open, then let a fresh document read them.
  const longName = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789ABCD';
  for (let index = 1; index <= 8; index++) {
    const id = index.toString(16).padStart(12, 'f');
    const folder = join(profilesRoot, id);
    mkdirSync(folder);
    writeFileSync(join(folder, 'profile.json'), JSON.stringify({
      version: 1, id, name: index === 1 ? longName : `Profile ${index + 2}`, created_at: work.created_at + index,
    }), { flag: 'wx' });
  }
  writeFileSync(launcherFile, JSON.stringify({ ask_every_time: true, last_profile: work.id }));
  await request('POST', `/session/${sessionId()}/refresh`, {});
  await until(async () => await visibleCards() === 10, 'all ten profiles are shown');
  await checkRows([3, 3, 3, 1]);
  assert.equal(await script(`const body = document.querySelector('.picker-body');
    body.scrollTop = body.scrollHeight;
    const preference = document.querySelector('[data-testid=profile-picker-ask]').getBoundingClientRect();
    return body.scrollHeight > body.clientHeight && preference.bottom <= body.getBoundingClientRect().bottom;`), true, 'the checkbox stays reachable by scrolling');
  await script('document.querySelector(".picker-body").scrollTop = 0; return true');
  await screenshot('profile-picker-ten-minimum');
  await resizeWindow(1280, 820);
  await until(() => script('return innerWidth >= 704'), 'picker is back at desktop width');
  await checkRows([4, 4, 2]);
  assert.equal(await script(`const name = Array.from(document.querySelectorAll('.picker-name')).find(item => item.textContent.trim() === arguments[0]);
    return name.title === arguments[0] && name.offsetHeight <= 2 * parseFloat(getComputedStyle(name).lineHeight) + 1;`, longName), true);
  assert.equal(await script(`return [...document.querySelectorAll('.picker-last')].map(badge => badge.closest('button').dataset.profileId).join(',')`), work.id);
  await screenshot('profile-picker-ten');

  // Choosing Work (and turning "always ask" off) starts that profile and ends the picker.
  await script('document.querySelector("[data-testid=profile-picker-ask]").click(); return true');
  assert.equal(await script('return document.querySelector("[data-testid=profile-picker-ask]").checked'), false);
  await script('document.querySelector(\'[data-profile-id="default"]\').focus(); return true');
  await press('\uE004');
  assert.equal(await script('return document.activeElement.dataset.profileId'), work.id, 'Tab follows the card order');
  assert.equal(await script('return document.activeElement.matches(":focus-visible")'), true);
  await press('\uE00D');
  await until(() => existsSync(join(profilesRoot, work.id, 'data', 'spark_browser.db')), 'the chosen profile started with its own database', 60_000);
  await until(() => existsSync(launcherFile), 'the choice was saved');
  assert.deepEqual(JSON.parse(readFileSync(launcherFile, 'utf8')), { ask_every_time: false, last_profile: work.id });
  // Choosing ended the picker process, so the driver session is gone on purpose.
  forgetSession();
  killProfile(work.id);
  await wait(1500);

  // The next start (no arguments) opens the remembered profile straight away, titled with its name.
  const { executable } = JSON.parse(readFileSync(resolve('.cache/native-target/test-build.json'), 'utf8'));
  const child = spawn(executable, [], { env: { ...process.env, SPARK_NATIVE_TEST_RUN_DIR: directory, SPARK_NATIVE_REAL_PROFILES: '1' }, stdio: 'ignore', windowsHide: false });
  try {
    if (process.platform === 'win32') {
      const title = () => spawnSync('powershell', ['-NoProfile', '-Command', `(Get-Process -Id ${child.pid} -ErrorAction SilentlyContinue).MainWindowTitle`], { encoding: 'utf8' }).stdout.trim();
      await until(() => title() === 'Spark - Work', 'a start with no arguments opened the remembered profile', 60_000);
    } else {
      await until(() => existsSync(join(profilesRoot, work.id, 'data', 'profile.lock')), 'the remembered profile was locked by the new start', 60_000);
    }
  } finally {
    if (process.platform === 'win32') spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { encoding: 'utf8' }); else child.kill('SIGKILL');
  }
  await wait(1000);
}
