import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { test } from 'node:test';
const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8').replace(/\r\n/g, '\n');

test('the WhatsApp command is registered in the handler, the ACL manifest and the shell capability', () => {
  assert.match(read('src-tauri/src/lib.rs'), /toggle_whatsapp_popover,?\n[\s\S]*?\]\)\)/);
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
  assert.match(read('src-tauri/capabilities/default.json'), /"webviews":\s*\[\s*"main",\s*"window-\*"\s*\]/);
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
  assert.match(actions, /invoke\("toggle_whatsapp_popover", \{ anchor, mode/);
  assert.match(read('src/lib/stores/titlebarActionsStore.ts'), /"whatsapp", "extensions"/);
});

test('WhatsApp display mode is saved and offers compact and side-panel layouts', () => {
  const settings = read('src/lib/stores/settingsStore.ts');
  assert.match(settings, /type WhatsAppDisplayMode = "popover" \| "side_panel"/);
  assert.match(settings, /spark_whatsapp_display_mode/);
  assert.match(settings, /saved === "side_panel" \? "side_panel" : "popover"/);

  const appearance = read('src/lib/components/settings/AppearanceSettings.svelte');
  assert.match(appearance, /setting-whatsapp-display-mode/);
  assert.match(appearance, /setWhatsAppDisplayMode\("popover"\)/);
  assert.match(appearance, /setWhatsAppDisplayMode\("side_panel"\)/);

  const popover = read('src-tauri/src/window/whatsapp_window.rs');
  assert.match(popover, /const WIDTH: f64 = 560\.0/);
  assert.match(popover, /SIDE_PANEL_WIDTH: f64 = 420\.0/);
  assert.match(popover, /side_panel_from_viewport/);
  assert.match(read('src-tauri/src/window/viewport.rs'), /whatsapp_window::resize\(app, bounds\)/);
  assert.match(read('src/lib/components/viewport/ViewportFrame.svelte'), /whatsapp-panel-slot/);
});

test('WhatsApp toolbar icon includes a monochrome handset and docs describe both modes', () => {
  const icon = read('src/lib/components/titlebar/TitlebarActionIcon.svelte');
  assert.match(icon, /action === "whatsapp"[\s\S]*?M8\.5 8\.1/);
  for (const file of ['docs/en/features/whatsapp-window.md', 'docs/pt-BR/features/whatsapp-window.md']) {
    const docs = read(file);
    assert.match(docs, /560 × 680/);
    assert.match(docs, /Painel lateral|Side panel/);
  }
});

test('WhatsApp feature docs exist in both languages', () => {
  assert.match(read('docs/en/features/whatsapp-window.md'), /popover/i);
  assert.match(read('docs/pt-BR/features/whatsapp-window.md'), /popover/i);
});
