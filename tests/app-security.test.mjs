import { readFileSync, readdirSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('every application command is ACL covered and only the local shell receives permissions', () => {
  const lib = read('src-tauri/src/lib.rs');
  const commands = [...lib.match(/generate_handler!\[([\s\S]*?)\]/)[1].matchAll(/\b([a-z][a-z_]+)\b/g)].map(match => match[1]);
  const manifest = [...read('src-tauri/src/commands/manifest.rs').matchAll(/"([a-z_]+)"/g)].map(match => match[1]);
  assert.deepEqual(manifest.sort(), commands.sort());
  const capability = JSON.parse(read('src-tauri/capabilities/default.json'));
  assert.equal(capability.windows, undefined);
  assert.equal(capability.remote, undefined);
  assert.equal(capability.local, true);
  assert.deepEqual(capability.webviews, ['main']);
  const testOnly = ['set_dark_mode_probe_scheme', 'set_minimum_font_probe_size'];
  assert.deepEqual(capability.permissions.filter(p => p.startsWith('allow-')).sort(), commands.filter(c => !testOnly.includes(c)).map(c => `allow-${c.replaceAll('_', '-')}`).sort());
  for (const command of testOnly) assert.ok(!capability.permissions.includes(`allow-${command.replaceAll('_', '-')}`));
  assert.match(lib, /commands::access::shell_only/);
});

test('internal browser events have an explicit WebView recipient and Windows uses Evergreen', () => {
  for (const file of readdirSync(new URL('../src-tauri/src/', import.meta.url), { recursive: true }).filter(file => file.endsWith('.rs'))) {
    if (file.replaceAll('\\', '/') === 'security/mod.rs') continue;
    assert.doesNotMatch(read(`src-tauri/src/${file}`), /\.emit\(|\.emit_to\(\s*"main"/, `unscoped event in ${file}`);
  }
  assert.match(read('src-tauri/src/security/mod.rs'), /EventTarget::Webview\s*\{\s*label: "main"/);
  assert.deepEqual(JSON.parse(read('src-tauri/tauri.conf.json')).bundle.windows.webviewInstallMode, { type: 'downloadBootstrapper', silent: true });
});

test('isolated native shell uses the same WebView2 extension environment as content views', () => {
  const production = JSON.parse(read('src-tauri/tauri.conf.json'));
  const native = JSON.parse(read('tests/native/tauri.e2e.conf.json'));
  assert.equal(production.app.windows[0].browserExtensionsEnabled, true);
  assert.equal(native.app.windows[0].browserExtensionsEnabled, true);
});

test('local assets use CSP without inline scripts or remote script sources', () => {
  const security = JSON.parse(read('src-tauri/tauri.conf.json')).app.security;
  for (const policy of [security.csp, security.devCsp]) {
    assert.match(policy, /script-src 'self';/);
    assert.match(policy, /object-src 'none'/);
    assert.match(policy, /base-uri 'self'/);
  }
  for (const page of ['workspace-switcher', 'bookmark-context-menu', 'media-popover', 'permission-popover', 'history-overlay']) {
    const html = read(`public/${page}.html`);
    assert.doesNotMatch(html, /<script\s*>/);
    assert.match(html, new RegExp(`src="\\./overlays/${page}\\.js"`));
  }
});
