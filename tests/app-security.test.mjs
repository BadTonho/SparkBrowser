import { readFileSync } from 'node:fs';
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
  assert.deepEqual(capability.permissions.filter(p => p.startsWith('allow-')).sort(), commands.map(c => `allow-${c.replaceAll('_', '-')}`).sort());
  assert.match(lib, /commands::access::shell_only/);
});

test('local assets use CSP without inline scripts or remote script sources', () => {
  const security = JSON.parse(read('src-tauri/tauri.conf.json')).app.security;
  for (const policy of [security.csp, security.devCsp]) {
    assert.match(policy, /script-src 'self';/);
    assert.match(policy, /object-src 'none'/);
    assert.match(policy, /base-uri 'self'/);
  }
  for (const page of ['workspace-switcher', 'bookmark-context-menu']) {
    const html = read(`public/${page}.html`);
    assert.doesNotMatch(html, /<script\s*>/);
    assert.match(html, new RegExp(`src="\\./overlays/${page}\\.js"`));
  }
});
