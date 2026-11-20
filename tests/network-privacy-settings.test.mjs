import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = (path) => readFileSync(path, 'utf8');
const settings = read('src/lib/components/settings/PrivacySecuritySettings.svelte');

test('the Tor session is hidden from the interface while it is deferred', () => {
  assert.doesNotMatch(settings, /network_tor|setting-network-tor/);
  for (const lang of ['en', 'pt-BR']) {
    assert.doesNotMatch(read(`src/lib/i18n/catalogs/${lang}/settings.json`), /network_tor/);
  }
  assert.doesNotMatch(read('src/lib/components/settings/settings-catalog.ts'), /id: "tor/);
});

test('HTTPS-only mode is a switch saved through the privacy settings', () => {
  assert.match(settings, /id="setting-network-https"/);
  assert.match(settings, /role="switch"[^>]*checked=\{settings\?\.https_only \?\? false\}/);
  assert.match(settings, /await setHttpsOnly\(enabled\)/);
  const bridge = read('src/lib/services/bridge/privacy-settings.ts');
  assert.match(bridge, /invoke\("set_https_only", \{ enabled \}\)/);
  assert.match(bridge, /invoke\("allow_http_once", \{ tabId, url \}\)/);
  assert.match(bridge, /https_only: false/);
});

test('DNS over HTTPS offers the providers, saves the choice and asks for a restart', () => {
  assert.match(settings, /id="setting-network-doh"/);
  for (const value of ['off', 'cloudflare', 'google']) assert.match(settings, new RegExp(`<option value="${value}">`));
  assert.match(settings, /restartForNetwork/);
  assert.match(settings, /\$networkStatus\.restartRequired/);
  const bridge = read('src/lib/services/bridge/network.ts');
  assert.match(bridge, /invoke<NetworkStatus>\("set_doh_provider", \{ provider \}\)/);
  assert.match(bridge, /invoke<ShutdownState>\("restart_for_network"\)/);
});

test('the secure-connection notice offers one HTTP visit for the blocked tab only', () => {
  const notice = read('src/lib/components/status/HttpsOnlyNotice.svelte');
  assert.match(notice, /listen<Blocked>\("https-only-blocked"/);
  assert.match(notice, /tab\.id === blocked!\.tab_id && tab\.is_active/);
  assert.match(notice, /allowHttpOnce\(blocked\.tab_id, blocked\.url\)/);
  assert.match(read('src/App.svelte'), /<HttpsOnlyNotice \/>/);
});

test('every new command is declared once and has its permission', () => {
  const lib = read('src-tauri/src/lib.rs');
  const manifest = read('src-tauri/src/commands/manifest.rs');
  const capability = JSON.parse(read('src-tauri/capabilities/default.json'));
  for (const command of ['set_https_only', 'allow_http_once', 'get_network_status', 'set_doh_provider', 'restart_for_network']) {
    assert.match(manifest, new RegExp(`"${command}"`));
    assert.ok(capability.permissions.includes(`allow-${command.replaceAll('_', '-')}`), command);
  }
  assert.match(lib, /network::apply_environment\(/);
  assert.match(lib, /network::start\(\)/);
});

test('the engines are pointed at the proxy before any WebView exists', () => {
  const lib = read('src-tauri/src/lib.rs');
  assert.ok(lib.indexOf('graphics::apply_environment(accelerated)') < lib.indexOf('network::apply_environment('));
  assert.ok(lib.indexOf('network::apply_environment(') < lib.indexOf('let builder = tauri::Builder::default()'));
  assert.match(read('src-tauri/src/window/mod.rs'), /crate::network::proxy_url\(\)/);
});

test('the network privacy documentation exists in both languages and says what is not covered', () => {
  for (const lang of ['en', 'pt-BR']) {
    const doc = read(`docs/${lang}/features/network-privacy.md`);
    assert.match(doc, /WebRTC/);
    assert.match(doc, /Tor/);
    assert.match(read(`docs/${lang}/README.md`), /features\/network-privacy\.md/);
  }
});
