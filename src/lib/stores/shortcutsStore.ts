import { UserError, describeError } from "../services/user-errors.js";
import { safeStorage } from "../services/storage.js";
import { get, writable } from "svelte/store";
import { setKeyboardShortcuts as syncKeyboardShortcuts } from "../services/tauri-bridge";
import { logIgnored } from "../services/ignored-error";

export type ShortcutCombo = {
  key: string;
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
};

export type ShortcutDefinition = {
  id: string;
  label: string;
  description: string;
  defaults: ShortcutCombo[];
  sequence?: "digit";
  hold?: boolean;
};

const combo = (key: string, modifiers: Partial<Omit<ShortcutCombo, "key">> = {}): ShortcutCombo => ({
  key,
  ctrl: modifiers.ctrl ?? false,
  alt: modifiers.alt ?? false,
  shift: modifiers.shift ?? false,
  meta: modifiers.meta ?? false,
});

export const SHORTCUT_DEFINITIONS: ShortcutDefinition[] = [
  { id: "next-tab", label: "Next tab", description: "Next tab in the focused pane", defaults: [combo("Tab", { ctrl: true })] },
  { id: "previous-tab", label: "Previous tab", description: "Previous tab in the focused pane", defaults: [combo("Tab", { ctrl: true, shift: true })] },
  { id: "zoom-in", label: "Zoom in", description: "Increase page zoom", defaults: [combo("+", { ctrl: true })] },
  { id: "zoom-out", label: "Zoom out", description: "Decrease page zoom", defaults: [combo("-", { ctrl: true })] },
  { id: "zoom-reset", label: "Reset page zoom", description: "Restore page zoom to 100%", defaults: [combo("0", { ctrl: true })] },
  { id: "reopen-tab", label: "Reopen closed tab", description: "Restore the most recently closed tab", defaults: [combo("T", { ctrl: true, shift: true })] },
  { id: "new-window", label: "Open new window", description: "Open another browser window with its own tabs", defaults: [combo("N", { ctrl: true })] },
  { id: "private-window", label: "Open private window", description: "Open a separate window that keeps no history, cookies or session", defaults: [combo("N", { ctrl: true, shift: true })] },
  { id: "switch-tabs", label: "Switch to Tab 1–9 (9: Last Tab)", description: "Switch tabs by number", defaults: [combo("Digit", { ctrl: true })], sequence: "digit" },
  { id: "workspace-picker", label: "Open the workspace picker", description: "Hold a modifier to show workspaces", defaults: [combo("Alt")], hold: true },
  { id: "switch-workspaces", label: "Switch workspaces by list order", description: "Switch to a workspace by number", defaults: [combo("Digit", { alt: true })], sequence: "digit" },
  { id: "move-tab-up", label: "Move Active Tab Up", description: "Move the active tab up", defaults: [combo("ArrowUp", { alt: true })] },
  { id: "move-tab-down", label: "Move Active Tab Down", description: "Move the active tab down", defaults: [combo("ArrowDown", { alt: true })] },
  { id: "move-tab-up-ctrl-shift", label: "Move Active Tab Up (alternate)", description: "Alternate shortcut to move the active tab up", defaults: [combo("ArrowUp", { ctrl: true, shift: true })] },
  { id: "move-tab-down-ctrl-shift", label: "Move Active Tab Down (alternate)", description: "Alternate shortcut to move the active tab down", defaults: [combo("ArrowDown", { ctrl: true, shift: true })] },
  { id: "new-tab", label: "Open a new tab", description: "Create a new tab", defaults: [combo("T", { ctrl: true })] },
  { id: "navigate-back", label: "Go Back", description: "Go to the previous page", defaults: [combo("Mouse4")] },
  { id: "navigate-forward", label: "Go Forward", description: "Go to the next page", defaults: [combo("Mouse5")] },
  { id: "command-bar", label: "Open Floating Command Bar", description: "Open the command bar", defaults: [combo("K", { ctrl: true })] },
    { id: "edit-url", label: "Edit Current URL in Address Bar", description: "Focus the current URL in the address bar", defaults: [combo("L", { ctrl: true })] },
  { id: "focus-address", label: "Focus Address and Search Bar", description: "Focus and select the current address or search text", defaults: [combo("E", { ctrl: true })] },
  { id: "sidebar", label: "Toggle Sidebar (Vertical Layout)", description: "Show or hide the sidebar", defaults: [combo("B", { ctrl: true })] },
  { id: "zen", label: "Toggle Fullscreen Zen / Focus Mode", description: "Toggle Zen mode", defaults: [combo("Z", { ctrl: true, shift: true })] },
  { id: "history", label: "Open SQLite Browsing History", description: "Open browsing history", defaults: [combo("H", { ctrl: true })] },
  { id: "downloads", label: "Open Download Manager", description: "Open downloads", defaults: [combo("J", { ctrl: true })] },
  { id: "settings", label: "Open Settings & Shortcuts Panel", description: "Open browser settings", defaults: [combo(",", { ctrl: true })] },
  { id: "close-tab", label: "Close Active Tab", description: "Close the active tab", defaults: [combo("W", { ctrl: true })] },
  { id: "reload-tab", label: "Reload Active Tab", description: "Reload the current page", defaults: [combo("R", { ctrl: true })] },
  { id: "toggle-pip", label: "Toggle Picture-in-Picture", description: "Pop out video from active tab into floating window", defaults: [combo("P", { alt: true })] },
];

export type ShortcutMap = Record<string, ShortcutCombo[]>;
const STORAGE_KEY = "spark_keyboard_shortcuts";

function defaultsMap(): ShortcutMap {
  return Object.fromEntries(SHORTCUT_DEFINITIONS.map(({ id, defaults }) => [id, defaults.map((item) => ({ ...item }))]));
}

function isCombo(value: unknown): value is ShortcutCombo {
  if (!value || typeof value !== "object") return false;
  const item = value as Partial<ShortcutCombo>;
  return typeof item.key === "string" && [item.ctrl, item.alt, item.shift, item.meta].every((flag) => typeof flag === "boolean");
}

function loadInitialShortcuts(): ShortcutMap {
  const fallback = defaultsMap();
  if (typeof safeStorage === "undefined") return fallback;
  try {
    const stored: unknown = JSON.parse(safeStorage.getItem(STORAGE_KEY) ?? "null");
    if (!stored || typeof stored !== "object") return fallback;
    const source = stored as Record<string, unknown>;
    for (const definition of SHORTCUT_DEFINITIONS) {
      const candidate = source[definition.id];
      const validKey = (binding: ShortcutCombo) => {
        if (definition.hold) return ["Alt", "Control", "Shift", "Meta"].includes(binding.key);
        if (definition.sequence) return binding.key === "Digit";
        return binding.key.length > 0 && binding.key.length <= 40;
      };
      if (Array.isArray(candidate) && candidate.length === 1 && candidate.every(isCombo) && validKey(candidate[0])) {
        fallback[definition.id] = candidate.map((item) => ({ ...item }));
      }
    }
    const oldZen = combo("Z", { alt: true });
    const newZen = combo("Z", { ctrl: true, shift: true });
    const newZenTaken = Object.entries(fallback).some(([id, bindings]) => id !== "zen" && bindings.some(binding => sameCombo(binding, newZen)));
    if (!Array.isArray(source.zen) && newZenTaken) {
      // A newly added default must not erase another saved customization.
      const oldZenTaken = Object.entries(fallback).some(([id, bindings]) => id !== "zen" && bindings.some(binding => sameCombo(binding, oldZen)));
      fallback.zen = oldZenTaken ? [] : [oldZen];
    }
    if (fallback.zen?.[0] && sameCombo(fallback.zen[0], oldZen) &&
        !newZenTaken) {
      fallback.zen = [newZen];
      try { safeStorage.setItem(STORAGE_KEY, JSON.stringify(fallback)); } catch { /* Keep the migrated shortcut for this session. */ }
    }
    const assigned: ShortcutCombo[] = [];
    for (const definition of SHORTCUT_DEFINITIONS) {
      const binding = fallback[definition.id]?.[0];
      if (!binding) continue;
      if (assigned.some((existing) => sameCombo(existing, binding))) return defaultsMap();
      assigned.push(binding);
    }
  } catch {
    // Ignore malformed or unavailable local storage and keep the defaults.
  }
  return fallback;
}

export const keyboardShortcuts = writable<ShortcutMap>(loadInitialShortcuts());
export const recordingShortcutId = writable<string | null>(null);

export function formatShortcut(binding: ShortcutCombo, sequence?: "digit"): string {
  const parts = [
    ...(binding.ctrl ? ["Ctrl"] : []),
    ...(binding.alt ? ["Alt"] : []),
    ...(binding.shift ? ["Shift"] : []),
    ...(binding.meta ? ["Meta"] : []),
  ];
  if (sequence === "digit") parts.push("1…9");
  else {
    const key = binding.key === "Control" ? "Ctrl" : binding.key === "ArrowUp" ? "↑" : binding.key === "ArrowDown" ? "↓" : binding.key === "Mouse4" ? "Mouse 4" : binding.key === "Mouse5" ? "Mouse 5" : binding.key;
    parts.push(key);
  }
  return parts.join(" + ");
}

export function shortcutLabel(id: string): string {
  const definition = SHORTCUT_DEFINITIONS.find((item) => item.id === id);
  if (!definition) return "";
  const binding = getShortcutBinding(keyboardShortcutsValue, id);
  return binding ? formatShortcut(binding, definition.sequence) : "Unassigned";
}

// The bound store value is mirrored synchronously for simple tooltip consumers.
let keyboardShortcutsValue = get(keyboardShortcuts);
keyboardShortcuts.subscribe((value) => {
  keyboardShortcutsValue = value;
});

export function getShortcutBinding(map: ShortcutMap, id: string): ShortcutCombo | undefined {
  return map[id]?.[0];
}

export function setShortcutBinding(id: string, binding: ShortcutCombo): string | undefined {
  const current = get(keyboardShortcuts);
  const definition = SHORTCUT_DEFINITIONS.find((item) => item.id === id);
  if (!definition) return describeError(new UserError("shortcut.invalid", "Unknown shortcut"));
  const candidate = { ...binding, key: definition.sequence === "digit" ? "Digit" : binding.key };
  const conflict = SHORTCUT_DEFINITIONS.find((item) => {
    if (item.id === id) return false;
    return (current[item.id] ?? []).some((existing) => sameCombo(existing, candidate));
  });
  if (conflict) return describeError(new UserError("shortcut.conflict", conflict.id));

  const next = { ...current, [id]: [candidate] };
  keyboardShortcuts.set(next);
  if (typeof safeStorage !== "undefined") {
    try { safeStorage.setItem(STORAGE_KEY, JSON.stringify(next)); } catch { /* Keep the in-memory setting. */ }
  }
  void syncKeyboardShortcuts(next).catch(logIgnored);
  return undefined;
}

function sameCombo(a: ShortcutCombo, b: ShortcutCombo): boolean {
  return a.key.toLowerCase() === b.key.toLowerCase() && a.ctrl === b.ctrl && a.alt === b.alt && a.shift === b.shift && a.meta === b.meta;
}

export function matchesShortcut(event: KeyboardEvent, binding: ShortcutCombo, sequence?: "digit"): boolean {
  if (sequence === "digit") {
    if (!/^[1-9]$/.test(event.key)) return false;
  } else if (event.key.toLowerCase() !== binding.key.toLowerCase() && !(binding.key === "+" && event.key === "=")) return false;
  const plusShift = binding.key === "+" && !binding.shift;
  return event.ctrlKey === binding.ctrl && event.altKey === binding.alt && (plusShift || event.shiftKey === binding.shift) && event.metaKey === binding.meta;
}

export function matchesMouseShortcut(event: MouseEvent, binding: ShortcutCombo): boolean {
  const key = event.button === 3 ? "Mouse4" : event.button === 4 ? "Mouse5" : undefined;
  return key === binding.key && event.ctrlKey === binding.ctrl && event.altKey === binding.alt &&
    event.shiftKey === binding.shift && event.metaKey === binding.meta;
}

export function initializeKeyboardShortcuts(): void {
  void syncKeyboardShortcuts(get(keyboardShortcuts)).catch(logIgnored);
}

export async function restoreShortcutMap(map: ShortcutMap): Promise<void> {
  const next = { ...get(keyboardShortcuts), ...map };
  const assigned: ShortcutCombo[] = [];
  for (const definition of SHORTCUT_DEFINITIONS) {
    const binding = next[definition.id]?.[0];
    if (!binding || assigned.some(item => sameCombo(item, binding))) throw new UserError("shortcut.conflict", "Invalid imported shortcut map");
    assigned.push(binding);
  }
  if (!safeStorage.setItem(STORAGE_KEY, JSON.stringify(next))) throw new UserError("storage.failed", "Keyboard preference write failed");
  await syncKeyboardShortcuts(next);
  keyboardShortcuts.set(next);
}

export function isWorkspacePickerHoldKey(event: KeyboardEvent): boolean {
  const binding = getShortcutBinding(get(keyboardShortcuts), "workspace-picker");
  return Boolean(binding && event.key === binding.key);
}
