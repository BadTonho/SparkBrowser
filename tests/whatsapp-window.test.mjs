import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('the WhatsApp command is registered in the handler, the ACL manifest and the shell capability', () => {
  assert.match(read('src-tauri/src/lib.rs'), /toggle_whatsapp_popover\n\s*\]\)\)/);
  assert.match(read('src-tauri/src/commands/manifest.rs'), /"toggle_whatsapp_popover",/);
  assert.match(read('src-tauri/capabilities/default.json'), /"allow-toggle-whatsapp-popover"/);
});

test('the WhatsApp child WebView is an external page without IPC that hides instead of closing', () => {
  const popover = read('src-tauri/src/window/whatsapp_window.rs');
  assert.match(popover, /WebviewUrl::External\(url\)/);
  assert.match(popover, /"https:\/\/web\.whatsapp\.com\/"/);
  assert.match(popover, /crate::security::harden\(app, &view, false\)/);
  assert.match(popover, /on_new_window\(\|_, _\| NewWindowResponse::Deny\)/);
  assert.match(popover, /view\.hide\(\)/);
  assert.match(read('src-tauri/capabilities/default.json'), /"webviews":\s*\[\s*"main"\s*\]/);
});

test('the popover module is Windows only and the command reports unsupported platforms', () => {
  assert.match(read('src-tauri/src/window/mod.rs'), /#\[cfg\(windows\)\]\npub mod whatsapp_window;/);
  const command = read('src-tauri/src/commands/whatsapp.rs');
  assert.match(command, /#\[cfg\(not\(windows\)\)\]/);
  assert.match(command, /Err\("whatsapp\.unsupported_platform"\.into\(\)\)/);
});

test('the titlebar button sends its own rectangle and stays hidden outside Windows', () => {
  const actions = read('src/lib/components/titlebar/TitlebarActions.svelte');
  assert.match(actions, /navigator\.userAgent\.includes\("Windows"\)/);
  assert.match(actions, /\(id !== "whatsapp" \|\| supportsWhatsApp\)/);
  assert.match(actions, /invoke\("toggle_whatsapp_popover", \{ anchor:/);
  assert.match(read('src/lib/stores/titlebarActionsStore.ts'), /"qr", "whatsapp"\] as const/);
});

test('WhatsApp feature docs exist in both languages', () => {
  assert.match(read('docs/en/features/whatsapp-window.md'), /popover/i);
  assert.match(read('docs/pt-BR/features/whatsapp-window.md'), /popover/i);
});
