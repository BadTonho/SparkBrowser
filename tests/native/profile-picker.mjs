import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { spawn, spawnSync } from 'node:child_process';

/**
 * The profile picker at start-up, in the real app. With two profiles and "ask every time" on, Spark
 * starts as a picker with no profile of its own: it answers only the picker commands, loads no browser,
 * and choosing a profile starts that profile in its own process, remembers the choice and ends the
 * picker. The next start then opens the remembered profile straight away.
 */
export async function checkProfilePicker(context) {
  const { invoke, until, script, directory, restart, screenshot, wait, forgetSession } = context;
  const profilesRoot = join(directory, 'profile', 'profiles');
  const launcherFile = join(directory, 'profile', 'launcher.json');
  const refusal = async (command, args) => {
    try { await invoke(command, args); return ''; } catch (error) { return String(error?.message ?? error); }
  };
  const killProfile = id => process.platform === 'win32' && spawnSync('powershell', ['-NoProfile', '-Command', `Get-CimInstance Win32_Process | Where-Object { $_.Name -like 'spark-browser*' -and $_.CommandLine -like '*--profile ${id}*' } | ForEach-Object { taskkill /PID $_.ProcessId /T /F 2>&1 | Out-Null }; exit 0`], { encoding: 'utf8' });

  // With a single profile there is nothing to choose; the toggle is saved for the next start.
  assert.deepEqual(await invoke('get_launch_state'), { mode: 'browser' });
  assert.deepEqual(await invoke('get_launcher_prefs'), { ask_every_time: false });
  const work = await invoke('create_profile', { name: 'Work' });
  await invoke('set_ask_profile_at_startup', { enabled: true });
  assert.deepEqual(await invoke('get_launcher_prefs'), { ask_every_time: true });
  assert.match(await refusal('get_picker_state'), /profile\.not_picker/, 'only the picker answers the picker commands');
  assert.match(await refusal('choose_profile', { id: work.id, askEveryTime: false }), /profile\.not_picker/);

  // The next start is the picker.
  await restart({ launcher: true });
  assert.deepEqual(await invoke('get_launch_state'), { mode: 'picker' });
  assert.equal((await invoke('get_diagnostics')).startup_ready, false, 'the picker never becomes the browser');
  assert.match(await refusal('get_tabs'), /start-up is still in progress|startup is still in progress/i, 'no browser command works in the picker');
  assert.match(await refusal('list_profiles'), /still in progress/i);
  const picker = await invoke('get_picker_state');
  assert.deepEqual(picker.profiles.map(profile => profile.name), ['', 'Work']);
  assert.equal(picker.ask_every_time, true);
  assert.equal(picker.last_profile, null, 'nothing was chosen yet');
  assert.equal(await script('return document.querySelectorAll("[data-testid=profile-picker-item]").length'), 2);
  assert.equal(await script('return document.querySelector("[data-testid=profile-picker-ask]").checked'), true);
  assert.equal(await script('return !!document.querySelector(".browser-root, .titlebar, .h-tabbar")'), false, 'none of the browser is on screen');
  await screenshot('profile-picker');

  // Choosing Work (and turning "always ask" off) starts that profile and ends the picker.
  await script('document.querySelector("[data-testid=profile-picker-ask]").click(); return true');
  assert.equal(await script('return document.querySelector("[data-testid=profile-picker-ask]").checked'), false);
  await script(`document.querySelector('[data-testid="profile-picker-item"][data-profile-id="${work.id}"]').click(); return true`);
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
