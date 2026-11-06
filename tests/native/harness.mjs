import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { resolve, toNamespacedPath } from 'node:path';
import { readFileSync, writeFileSync } from 'node:fs';
import { createFixture } from './fixture.mjs';
import { cleanRunData, createRunDirectory } from './run-directory.mjs';

export async function runScenario(name, scenario) {
  if (!['win32', 'linux'].includes(process.platform)) throw new Error('This direct tauri-driver suite supports Windows/Linux. Use an embedded WebDriver service on macOS.');
  const build = JSON.parse(readFileSync('.cache/native-target/test-build.json', 'utf8'));
  assert.equal(build.identifier, 'org.veloce.browser.native-tests', 'Build the isolated native test application first');
  const binary = resolve(build.executable);
  assert.ok(binary.startsWith(resolve('.cache/native-target') + (process.platform === 'win32' ? '\\' : '/')));
  const directory = createRunDirectory(name);
  const port = Number(process.env.VELOCE_DRIVER_PORT ?? 4444);
  const endpoint = `http://127.0.0.1:${port}`;
  const driverPath = process.env.VELOCE_TAURI_DRIVER ?? resolve('.cache/native-tools/bin', process.platform === 'win32' ? 'tauri-driver.exe' : 'tauri-driver');
  const nativePath = process.env.VELOCE_NATIVE_DRIVER;
  const args = ['--port', String(port), '--native-port', String(port + 1)];
  if (nativePath) args.push('--native-driver', resolve(nativePath));
  let driver;
  let driverError;

  let session;
  let shellWebview;
  let fixture;
  const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
  async function until(check, message, timeout = 20_000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      if (driverError) throw driverError;
      const value = await check();
      if (value) return value;
      await wait(100);
    }
    throw new Error(`Timed out: ${message}`);
  }
  async function request(method, path, body) {
    const result = await fetch(endpoint + path, { method, headers: { 'Content-Type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body), signal: AbortSignal.timeout(60_000) });
    const payload = await result.json();
    if (!result.ok || (typeof payload.value?.error === 'string' && typeof payload.value?.message === 'string')) throw new Error(`WebDriver ${path}: ${payload.value?.message ?? result.status}`);
    return payload.value;
  }
  async function script(code, ...args) { return request('POST', `/session/${session}/execute/sync`, { script: code, args }); }
  async function click(selector) {
    await until(() => script('return Boolean(document.querySelector(arguments[0]))', selector), `control available: ${selector}`);
    const projected = await script('const node=document.querySelector(arguments[0]);return node?.closest(".tools-controller")?"tools":node?.closest(".downloads-controller")?"downloads":null', selector);
    if (projected) return withWebview(`/index.html?surface=${projected}`, async () => {
      await until(() => script('return Boolean(document.querySelector(arguments[0]))', selector), 'projected panel control');
      const element = await request('POST', `/session/${session}/element`, { using: 'css selector', value: selector });
      await request('POST', `/session/${session}/element/${element['element-6066-11e4-a52e-4f735466cecf']}/click`, {});
    });
    const element = await request('POST', `/session/${session}/element`, { using: 'css selector', value: selector });
    await request('POST', `/session/${session}/element/${element['element-6066-11e4-a52e-4f735466cecf']}/click`, {});
  }
  const webviewHandles = new Map();
  async function withWebview(path, operation) {
    const previous = await request('GET', `/session/${session}/window`);
    const started = Date.now();
    let polls = 0;
    try {
      await until(async () => {
        polls++;
        const handles = await request('GET', `/session/${session}/window/handles`);
        const cached = webviewHandles.get(path);
        const ordered = [...new Set([cached, ...handles.slice().reverse()].filter(handle => handle && handles.includes(handle)))];
        for (const handle of ordered) {
          try {
            await request('POST', `/session/${session}/window`, { handle });
            const candidateUrl = new URL(await request('GET', `/session/${session}/url`));
            if ((path.includes('?') ? candidateUrl.pathname + candidateUrl.search : candidateUrl.pathname) === path) {
              webviewHandles.set(path, handle);
              return true;
            }
          } catch (error) {
            if (!/no such window|web view not found/i.test(String(error))) throw error;
          }
        }
        await request('POST', `/session/${session}/window`, { handle: previous });
        return false;
      }, `Native WebView ready: ${path} (creation and navigation)`, 10_000);
      if (polls > 1) console.log(`Native WebView ready: ${path}, ${Date.now() - started} ms, ${polls} polls`);
      return await operation();
    } finally {
      await request('POST', `/session/${session}/window`, { handle: previous });
    }
  }
  async function windowsWindowAction(args) {
    await new Promise((resolveAction, reject) => {
      const helper = spawn('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve('tests/native/windows-window.ps1'),
        '-Executable', binary, ...args], { windowsHide: true, stdio: ['ignore', 'ignore', 'pipe'] });
      let error = '';
      helper.stderr.on('data', chunk => { error += chunk; });
      helper.on('error', reject);
      helper.on('exit', code => code === 0 ? resolveAction() : reject(new Error(error || 'Native window operation failed')));
    });
  }
  async function resizeWindow(width, height) {
    if (process.platform !== 'win32') return request('POST', `/session/${session}/window/rect`, { width, height });
    await windowsWindowAction(['-Width', String(width), '-Height', String(height)]);
  }
  async function focusWindow() {
    if (process.platform === 'win32') {
      for (let attempt = 0; attempt < 3; attempt++) {
        await invoke('plugin:window|set_focus', { label:'main' });
        try { await windowsWindowAction(['-Focus']); return; }
        catch (error) {
          if (attempt === 2 || !String(error).includes('did not receive foreground focus')) throw error;
          await wait(150);
        }
      }
    }
  }
  async function floatingWindow(action='Inspect',name='pip-window') {
    if (['Drag', 'Resize', 'Capture'].includes(action)) {
      // Activate through the privileged shell before the OS gesture. Selecting
      // a WebDriver document alone does not give the native popup foreground focus.
      const previous = await request('GET', `/session/${session}/window`);
      try {
        let label;
        for (const handle of await request('GET', `/session/${session}/window/handles`)) {
          await request('POST', `/session/${session}/window`, { handle });
          const url = await request('GET', `/session/${session}/url`);
          const id = /^about:blank#veloce-pip-([a-f0-9]{64})$/.exec(url)?.[1];
          if (id) { label = `floating-video-${id}`; break; }
        }
        assert.ok(label && shellWebview, 'Only the isolated floating player can receive focus');
        await request('POST', `/session/${session}/window`, { handle: shellWebview });
        await invoke('plugin:window|set_focus', { label });
      } finally {
        await request('POST', `/session/${session}/window`, { handle: previous });
      }
    }
    return new Promise((resolveAction,reject)=>{
      const args=['-NoProfile','-ExecutionPolicy','Bypass','-File',resolve('tests/native/floating-window.ps1'),'-Executable',binary,'-Action',action];
      if(action==='Capture'){assert.match(name,/^[a-z0-9-]+$/);args.push('-ScreenshotPath',resolve(directory,'artifacts',name+'.png'));}
      const helper=spawn('powershell.exe',args,{windowsHide:true,stdio:['ignore','pipe','pipe']});
      let output='',error='';
      helper.stdout.on('data',chunk=>{output+=chunk;});helper.stderr.on('data',chunk=>{error+=chunk;});
      helper.on('error',reject);helper.on('exit',code=>{if(code!==0)reject(new Error(error||'Floating window inspection failed'));else {try{resolveAction(JSON.parse(output));}catch(e){reject(e);}}});
    });
  }
  async function invoke(command, args = {}) {
    const result = await request('POST', `/session/${session}/execute/async`, {
      script: `const done = arguments[arguments.length - 1]; window.__TAURI_INTERNALS__.invoke(arguments[0], arguments[1]).then(value => done({ value }), error => done({ error: String(error) }));`,
      args: [command, args],
    });
    if (result.error) throw new Error(`${command}: ${result.error}`);
    return result.value;
  }
  async function openSession() {
    const options = { application: binary };
    if (process.platform === 'win32') options.webviewOptions = { userDataFolder: resolve(directory, 'profile'), additionalBrowserArguments: ['--use-fake-device-for-media-stream', ...(name.startsWith('media') ? ['--autoplay-policy=no-user-gesture-required'] : [])] };
    const result = await request('POST', '/session', { capabilities: { alwaysMatch: { 'tauri:options': options } } });
    session = result.sessionId;
    // A local new-tab child can become WebDriver's initial target. Select the shell explicitly.
    await until(async () => {
      const handles = await request('GET', `/session/${session}/window/handles`);
      for (const handle of handles) {
        try {
          await request('POST', `/session/${session}/window`, { handle });
          if (await script('return Boolean(window.__TAURI_INTERNALS__) && Boolean(document.querySelector(".browser-root"))')) { shellWebview = handle; return true; }
        } catch (error) {
          if (!/no such window|web view not found/i.test(String(error))) throw error;
        }
      }
      return false;
    }, 'isolated test shell');
    await request('POST', `/session/${session}/timeouts`, { script: 30_000, implicit: 0, pageLoad: 30_000 });
    // Never operate on the user's production profile if a wrong binary was supplied.
    const diagnostics = await invoke('get_diagnostics');
    assert.equal(diagnostics.application_identifier, 'org.veloce.browser.native-tests');
    assert.equal(toNamespacedPath(resolve(diagnostics.application_data_directory)).toLowerCase(), toNamespacedPath(resolve(directory, 'data')).toLowerCase());
    await until(async () => (await invoke('get_diagnostics')).startup_ready, 'native startup completes');
    await until(() => script('return document.querySelectorAll(".h-tab,.tab-row").length > 0'), 'shell initialization');
  }
  async function closeSession() {
    if (!session) return;
    const id = session; session = undefined;
    await request('DELETE', `/session/${id}`);
  }

  let failure;
  const cleanupErrors = [];
  const started = new Date().toISOString();
  async function screenshot(name) {
    const png = await request('GET', `/session/${session}/screenshot`);
    writeFileSync(resolve(directory, 'artifacts', `${name}.png`), Buffer.from(png, 'base64'));
  }
  async function startDriver() {
    driverError = undefined;
    driver = spawn(driverPath, args, { stdio: 'ignore', windowsHide: true, env: { ...process.env, VELOCE_NATIVE_TEST_RUN_DIR: directory }, detached: process.platform !== 'win32' });
    driver.on('error', error => { driverError = error; });
    driver.on('exit', (code, signal) => { driverError = new Error(`Scenario driver exited (${code ?? signal})`); });
    await until(async () => { try { await request('GET', '/status'); return true; } catch { return false; } }, 'driver ready');
  }
  async function stopDriver() {
    if (driver?.pid) {
      const stopped = driver.exitCode !== null || driver.signalCode !== null ? Promise.resolve() : new Promise(resolve => driver.once('exit', resolve));
      if (process.platform === 'win32') {
        // Terminate only the child tree created for this scenario.
        await new Promise(resolve => { const kill = spawn('taskkill', ['/PID', String(driver.pid), '/T', '/F'], { windowsHide: true, stdio: 'ignore' }); kill.on('error', resolve); kill.on('close', resolve); });
      } else { try { process.kill(-driver.pid, 'SIGKILL'); } catch { /* Already exited. */ } }
      await stopped;
      // Windows can retain WebView profile/WAL handles briefly after process termination.
      await wait(250);
    }
    driver = undefined;
  }
  async function restart({ crash = false, beforeStart, expectFailure = false } = {}) {
    if (session && !crash) {
      await invoke('get_history', { limit: 10 });
      await script(`window.__TAURI_INTERNALS__.invoke('plugin:window|close', { label: 'main' }); return null;`).catch(() => {});
      await until(async () => {
        try {
          const state = await invoke('get_shutdown_state');
          if (state.phase === 'review') await invoke('confirm_shutdown', { attempt: state.attempt, revision: state.review.revision });
          if (state.phase === 'error') throw new Error('Protected shutdown failed: ' + state.error);
          return !(await request('GET', `/session/${session}/window/handles`)).length;
        } catch (error) { if (String(error).includes('Protected shutdown failed:')) throw error; return true; }
      }, 'normal native exit');
      await closeSession().catch(() => {});
    }
    // Forced restarts kill only our process tree, without sending a clean-close command.
    await stopDriver();
    session = undefined;
    await beforeStart?.(directory);
    await startDriver();
    if (!expectFailure) return openSession();
    let failure;
    try { await openSession(); } catch (error) { failure = error; }
    await stopDriver(); session = undefined;
    assert.ok(failure, 'The injected startup failure must prevent application readiness');
    return failure;
  }
  try {
    fixture = await createFixture();
    await startDriver();
    await openSession();
    await scenario({ invoke, until, fixture, script, click, wait, screenshot, request, withWebview, resizeWindow, focusWindow, floatingWindow, sessionId: () => session, restart, directory });
    if (process.env.VELOCE_NATIVE_FAIL_CASE === name) throw new Error('Intentional failure to verify finally cleanup');
  } catch (error) {
    failure = error;
    if (session) await screenshot('failure').catch(() => {});
  } finally {
    await closeSession().catch(error => cleanupErrors.push(String(error)));
    await stopDriver();
    await fixture?.close().catch(error => cleanupErrors.push(String(error)));
    await cleanRunData(directory).catch(error => cleanupErrors.push(String(error)));
    try {
      writeFileSync(resolve(directory, 'artifacts/report.json'), JSON.stringify({ name, started, finished: new Date().toISOString(), passed: !failure && !cleanupErrors.length, failure: failure?.stack, cleanupErrors }, null, 2));
    } catch (error) { cleanupErrors.push(String(error)); }
  }
  if (failure) {
    if (cleanupErrors.length) console.error(`Additional cleanup errors: ${cleanupErrors.join('; ')}`);
    throw failure;
  }
  if (cleanupErrors.length) throw new Error(`Scenario cleanup failed: ${cleanupErrors.join('; ')}`);
  console.log(`Native scenario passed: ${name} (${directory})`);

}
