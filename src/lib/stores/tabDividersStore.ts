import { get, writable } from "svelte/store";
import { logIgnored } from "../services/ignored-error";
import { isTauriRuntime } from "../services/bridge/runtime";
import { fetchTabDividers, saveTabDividers } from "../services/bridge/tab-dividers";

/** A named line in the vertical tab list. `after` is the tab it sits below ("" = top of the list). */
export interface TabDivider { id: string; after: string; name: string }
const LEGACY_KEY = "spark.tabDividers";
const MAX_NAME = 40;

export const tabDividers = writable<TabDivider[]>([]);
/** The divider whose name is being typed right now. */
export const editingDividerId = writable<string | null>(null);

// Dividers live in the app database. Nothing is written until the saved list has been read, so an
// early change can never overwrite what was saved.
let loaded = false;
let timer: ReturnType<typeof setTimeout> | undefined;
function flush(): void {
  timer = undefined;
  if (!isTauriRuntime()) return;
  saveTabDividers(get(tabDividers)).catch(logIgnored);
}
tabDividers.subscribe(() => {
  if (!loaded) return;
  clearTimeout(timer);
  timer = setTimeout(flush, 150);
});

/** Dividers saved by the first version, which kept only tab ids in the web view's storage. */
function legacyDividers(): TabDivider[] {
  try {
    const value = JSON.parse(localStorage.getItem(LEGACY_KEY) ?? "[]");
    if (!Array.isArray(value)) return [];
    return value.flatMap((item, index): TabDivider[] =>
      typeof item === "string" ? [{ id: `legacy-${index}`, after: item, name: "" }]
      : item && typeof item.id === "string" && typeof item.after === "string"
        ? [{ id: item.id.slice(0, 64), after: item.after, name: typeof item.name === "string" ? item.name.slice(0, MAX_NAME) : "" }] : []);
  } catch (error) { logIgnored(error); return []; }
}

export async function loadTabDividers(): Promise<void> {
  if (loaded) return;
  let saved: TabDivider[] = [];
  try { if (isTauriRuntime()) saved = await fetchTabDividers(); }
  catch (error) { logIgnored(error); return; } // Stay unloaded: nothing is saved over what could not be read.
  const pending = get(tabDividers);
  const items = saved.length ? saved : legacyDividers();
  const known = new Set(items.map(item => item.id));
  tabDividers.set([...items, ...pending.filter(item => !known.has(item.id))]);
  loaded = true;
  if (!saved.length && get(tabDividers).length) flush();
  try { localStorage.removeItem(LEGACY_KEY); } catch (error) { logIgnored(error); }
}

export function addTabDivider(after: string): string {
  const id = crypto.randomUUID();
  tabDividers.update(items => [...items, { id, after, name: "" }]);
  editingDividerId.set(id);
  return id;
}
export const removeTabDivider = (id: string) => tabDividers.update(items => items.filter(item => item.id !== id));
export const renameTabDivider = (id: string, name: string) =>
  tabDividers.update(items => items.map(item => item.id === id ? { ...item, name: name.trim().slice(0, MAX_NAME) } : item));
export const moveTabDivider = (id: string, after: string) =>
  tabDividers.update(items => items.some(item => item.id === id && item.after !== after) ? items.map(item => item.id === id ? { ...item, after } : item) : items);
/** A tab the user closed: its dividers move up to the tab above it (or the top of the list). */
export function reanchorTabDividers(closedTabId: string, newAfter: string): void {
  tabDividers.update(items => items.some(item => item.after === closedTabId) ? items.map(item => item.after === closedTabId ? { ...item, after: newAfter } : item) : items);
}
