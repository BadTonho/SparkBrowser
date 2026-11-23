import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

test("the command bar is a layer above the page and no longer hides the page", () => {
  assert.match(read("src/lib/services/bridge/overlays.ts"), /nonBlockingOverlayIds = new Set<OverlayId>\(\[[^\]]*"command-bar"/);
  const frame = read("src/lib/components/viewport/ViewportFrame.svelte");
  const blocking = frame.slice(frame.indexOf("let blockingOverlayOpen"), frame.indexOf("const viewportSync"));
  assert.ok(!blocking.includes("isCommandBarOpen"), "the placeholder card is not shown behind the bar");
});

test("the native layer is registered end to end", () => {
  const rust = read("src-tauri/src/window/panel_popover/mod.rs");
  assert.ok(rust.includes('const HOST: &\'static str = "commands-popover"') && rust.includes('"/index.html?surface=commands"'));
  assert.ok(rust.includes('"commands" => warm::<Commands>'), "it can be prepared ahead of the shortcut");
  for (const file of ["src-tauri/src/lib.rs", "src-tauri/src/commands/manifest.rs", "src-tauri/src/commands/access.rs", "src-tauri/capabilities/default.json"]) {
    assert.match(read(file), /set_commands_popover|allow-set-commands-popover/, file);
  }
  assert.ok(read("src/main.ts").includes('"commands"') && read("src/main.ts").includes("CommandsSurface.svelte"));
  const surface = read("src/lib/components/panels/PanelSurface.svelte");
  assert.ok(surface.includes("__sparkSetCommandsPopover") && surface.includes('".command-input"'));
});

test("the arrow keys and Enter of the layer reach the real bar, which keeps the logic", () => {
  const surface = read("src/lib/components/panels/PanelSurface.svelte");
  assert.match(surface, /\["ArrowDown", "ArrowUp", "Enter"\]\.includes\(event\.key\)/);
  assert.match(surface, /kind: "keyboard", target: Number\(target\.dataset\.panelControl\)/);
  const bar = read("src/lib/components/commandbar/CommandBar.svelte");
  assert.ok(bar.includes("mountPanelHost") && bar.includes('id: "commands"') && bar.includes('class="command-controller"'));
  assert.ok(bar.includes("beginCommandBarSession"));
});

test("the bar uses the browser's own surface instead of its own blue tint and glow", () => {
  const css = read("src/lib/components/commandbar/commandbar.css");
  for (const rule of [".command-modal", ".command-item.selected", ".commands-dismiss", ".command-footer"]) assert.ok(css.includes(rule), rule);
  assert.ok(css.includes("var(--modal-surface-bg") && css.includes("var(--modal-surface-border") && css.includes("var(--modal-surface-shadow"));
  assert.equal(/rgba\(20, 24, 38/.test(css), false, "no navy tint");
  assert.equal(/0 0 40px/.test(css), false, "no accent glow");
  assert.equal(/translateX\(3px\)/.test(css), false, "no sliding selected row");
  assert.equal(read("src/lib/components/commandbar/CommandBar.svelte").includes("<style>"), false, "the styles are shared, not scoped to one copy");
});

test("closing the bar gives the keyboard back to the browser window", () => {
  const store = read("src/lib/stores/commandBarStore.ts");
  assert.match(store, /closeCommandBar\(restoreFocus = true\)/);
  assert.match(store, /setCommandsPopover\(false, session\.id, undefined, restoreFocus\)/);
  assert.match(store, /await Promise\.all\(\[endCommandBarSession\(restoreFocus\), setOverlayVisibility\(false, "command-bar"\)\]\)/);
  const bar = read("src/lib/components/commandbar/CommandBar.svelte");
  assert.match(bar, /openSavedNote\(note\.url, \(\) => closeCommandBar\(false\)\)/);
  assert.match(bar, /openSavedSession\(session\.id, \(\) => closeCommandBar\(false\)\)/);
});
