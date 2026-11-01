import assert from 'node:assert/strict';
import { checkFeatures } from '../../tests/native/features.mjs';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { readFileSync } from 'node:fs';
import { createFixture } from '../../tests/native/fixture.mjs';

if (!['win32', 'linux'].includes(process.platform)) throw new Error('This direct tauri-driver suite supports Windows/Linux. Use an embedded WebDriver service on macOS.');
const build = JSON.parse(readFileSync('.cache/native-target/test-build.json', 'utf8'));
assert.equal(build.identifier, 'org.veloce.browser.native-tests', 'Build the isolated native test application first');
const binary = resolve(build.executable);
assert.ok(binary.startsWith(resolve('.cache/native-target') + (process.platform === 'win32' ? '\\' : '/')));
const port = Number(process.env.VELOCE_DRIVER_PORT ?? 4444);
const endpoint = `http://127.0.0.1:${port}`;
const driverPath = process.env.VELOCE_TAURI_DRIVER ?? resolve('.cache/native-tools/bin', process.platform === 'win32' ? 'tauri-driver.exe' : 'tauri-driver');
const nativePath = process.env.VELOCE_NATIVE_DRIVER;
const args = ['--port', String(port), '--native-port', String(port + 1)];
if (nativePath) args.push('--native-driver', resolve(nativePath));
const driver = spawn(driverPath, args, { stdio: 'ignore', windowsHide: true });
let driverError;
driver.on('error', error => { driverError = error; });
let session;
const fixture = await createFixture();
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
  if (!result.ok || payload.value?.error) throw new Error(`WebDriver ${path}: ${payload.value?.message ?? result.status}`);
  return payload.value;
}
async function script(code, ...args) { return request('POST', `/session/${session}/execute/sync`, { script: code, args }); }
async function click(selector) {
  const element = await request('POST', `/session/${session}/element`, { using: 'css selector', value: selector });
  await request('POST', `/session/${session}/element/${element['element-6066-11e4-a52e-4f735466cecf']}/click`, {});
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
  const result = await request('POST', '/session', { capabilities: { alwaysMatch: { 'tauri:options': { application: binary } } } });
  session = result.sessionId;
  await until(() => script('return Boolean(window.__TAURI_INTERNALS__) && Boolean(document.querySelector(".browser-root"))'), 'isolated test shell');
  await request('POST', `/session/${session}/timeouts`, { script: 30_000, implicit: 0, pageLoad: 30_000 });
  // Never operate on the user's production profile if a wrong binary was supplied.
  assert.equal((await invoke('get_diagnostics')).application_identifier, 'org.veloce.browser.native-tests');
  await until(() => script('return document.querySelectorAll(".h-tab,.tab-row").length > 0'), 'shell initialization');
}
async function closeSession() {
  if (!session) return;
  const id = session; session = undefined;
  await request('DELETE', `/session/${id}`);
}

try {
  await until(async () => { try { await request('GET', '/status'); return true; } catch { return false; } }, 'tauri-driver ready');
  await openSession();
  await invoke('set_resident_limit', { limit: 8 });
  const a = (await invoke('create_tab', { url: fixture.origin + '/page/a', searchEngine: 'google' })).tabs.find(tab => tab.is_active);
  await until(() => fixture.reports.get('a')?.value === 'artificial-test-draft', 'fixture form report');
  await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id)?.activity.dirty_form, 'edited form protection');
  fixture.commands.set('a', 'spa');
  await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id)?.url.endsWith('/spa'), 'SPA address update');
  const b = (await invoke('create_tab', { url: fixture.origin + '/page/b', searchEngine: 'google' })).tabs.find(tab => tab.is_active);
  await until(() => fixture.reports.has('b'), 'second page');
  for (let i = 0; i < 8; i++) await invoke('switch_tab', { tabId: i % 2 ? b.id : a.id });
  await invoke('switch_tab', { tabId: a.id });
  assert.equal(fixture.loads.get('a'), 1, 'switching resident tabs must not reload forms');
  assert.equal(fixture.reports.get('a').value, 'artificial-test-draft');
  assert.ok(fixture.reports.get('a').scroll >= 500, 'switching tabs must preserve scroll position');
  await invoke('set_tab_pinned', { tabId: b.id, pinned: true });
  await invoke('set_resident_limit', { limit: 1 });
  const protectedTabs = (await invoke('get_tabs')).tabs;
  assert.equal(protectedTabs.find(tab => tab.id === b.id).is_hibernated, false);
  assert.equal(protectedTabs.find(tab => tab.id === a.id).is_hibernated, false);
  await invoke('set_resident_limit', { limit: 8 });

  if (process.platform === 'win32') {
    await invoke('toggle_adblock_shield');
    assert.equal(fixture.loads.get('a'), 1, 'live shield changes must preserve the document');
    await invoke('go_back');
    await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id)?.url.endsWith('/page/a'), 'native back after shield update');
    await invoke('go_forward');
    await until(async () => (await invoke('get_tabs')).tabs.find(tab => tab.id === a.id)?.url.endsWith('/spa'), 'native forward after shield update');
    await invoke('toggle_adblock_shield');
  }

  // Draft and focus lifecycle are checked in the shell's DOM through its real event handlers.
  await until(() => script('return document.querySelector(".omnibox-input").value === arguments[0]', fixture.origin + '/page/a/spa'), 'frontend navigation settled');
  await click('.omnibox-input');
  await script(`const input = document.querySelector('.omnibox-input'); input.focus(); input.value = 'draft survives'; input.dispatchEvent(new Event('input', { bubbles: true }));`);
  assert.equal(await script('return document.activeElement === document.querySelector(".omnibox-input")'), true, 'address has focus');
  await invoke('reorder_tabs', { tabIds: [b.id, a.id] });
  await wait(150);
  assert.equal(await script('return document.querySelector(".omnibox-input").value'), 'draft survives');
  await script(`document.querySelector('.omnibox-input').dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));`);
  // Exercise real UI handlers without introducing test-only IPC or expanded permissions.
  for (const label of ['Settings', 'Downloads', 'Browsing History']) {
    await script(`const button = document.querySelector('button[aria-label="' + arguments[0] + '"]'); button.focus(); button.click();`, label);
    await until(() => script('return Boolean(document.querySelector("[role=dialog]"))'), `${label} dialog`);
    await until(() => script('return document.querySelector("[role=dialog]").contains(document.activeElement)'), 'initial dialog focus');
    assert.equal(await script(`const dialog = document.querySelector('[role=dialog]');
      const controls = [...dialog.querySelectorAll('input:not([disabled]),select:not([disabled]),textarea:not([disabled]),button:not([disabled]),a[href],[tabindex="0"]')].filter(node => node.getClientRects().length);
      controls.at(-1).focus(); document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }));
      return document.activeElement === controls[0];`), true, 'Tab wraps within the current dialog');
    await script('document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }))');
    await until(() => script('return !document.querySelector("[role=dialog]")'), 'Escape closes dialog');
    await until(() => script('return document.activeElement?.getAttribute("aria-label") === arguments[0]', label), 'return focus to opener');
  }
  await checkFeatures(invoke, until, fixture, a, b);
  for (const [width, height] of [[900, 700], [1400, 900]]) {
    await request('POST', `/session/${session}/window/rect`, { width, height });
    await until(() => script('const box = document.querySelector(".viewport-frame").getBoundingClientRect(); return box.width >= 100 && box.height >= 100;'), 'viewport after resize');
  }

  await invoke('get_history', { limit: 10 }); // Flush navigation writes before restart.
  await closeSession();
  await openSession();
  const restoredPacket = await invoke('get_tabs');
  const restored = restoredPacket.tabs;
  assert.equal(restoredPacket.split_layout.ratio, 0.65);
  assert.equal(restoredPacket.split_layout.left_tab_id, a.id);
  assert.ok(restored.some(tab => tab.id === a.id));
  assert.equal(restored.find(tab => tab.id === b.id)?.is_pinned, true);
  assert.ok(restored.filter(tab => !tab.is_active).every(tab => tab.is_hibernated || tab.is_visible));
  console.log('Native smoke passed: tabs, SPA history, form state, protected memory, draft, dialogs, resize and restoration.');
} finally {
  await closeSession().catch(() => {});
  driver.kill();
  await fixture.close();
}
