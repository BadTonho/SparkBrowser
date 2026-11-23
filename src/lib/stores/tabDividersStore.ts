import { writable } from "svelte/store";
import { logIgnored } from "../services/ignored-error";
import { isTauriRuntime } from "../services/bridge/runtime";
import { editTabDivider, fetchTabDividers } from "../services/bridge/tab-dividers";

/** A named line in the vertical tab list. `after` is the tab it sits below ("" = top of the list). */
export interface TabDivider { id: string; after: string; name: string }
type TabAction =
  | { type: "add"; divider: TabDivider }
  | { type: "rename"; id: string; name: string }
  | { type: "move"; id: string; after: string }
  | { type: "remove"; id: string };
export type DividerEdit = TabAction | { type: "import"; dividers: TabDivider[] };
const LEGACY_KEY = "spark.tabDividers";
const MAX_NAME = 40;

export const tabDividers = writable<TabDivider[]>([]);
/** The divider whose name is being typed right now. */
export const editingDividerId = writable<string | null>(null);

let saved: TabDivider[] = [];
let work = Promise.resolve();
const pending: TabAction[] = [];
let migrated = false;

function apply(items: TabDivider[], action: TabAction): TabDivider[] {
  if (action.type === "add") return [...items, action.divider];
  if (action.type === "remove") return items.filter(item => item.id !== action.id);
  const updated = action.type === "rename" ? { name: action.name } : { after: action.after };
  return items.map(item => item.id === action.id ? { ...item, ...updated } : item);
}

function render(): void {
  tabDividers.set(pending.reduce(apply, saved));
}

function enqueue(task: () => Promise<void>): Promise<void> {
  work = work.then(task).catch(logIgnored);
  return work;
}

function update(action: TabAction): void {
  if (!isTauriRuntime()) {
    saved = apply(saved, action);
    render();
    return;
  }
  pending.push(action);
  render();
  void enqueue(async () => {
    try {
      saved = await editTabDivider(action);
    } finally {
      pending.shift();
      render();
    }
  });
}

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

export function refreshTabDividers(migrate = false): Promise<void> {
  return enqueue(async () => {
    if (isTauriRuntime()) {
      saved = await fetchTabDividers();
      render();
    }
    if (!migrate || migrated) return;
    const dividers = legacyDividers();
    if (dividers.length) {
      saved = isTauriRuntime() ? await editTabDivider({ type: "import", dividers }) : dividers;
      render();
    }
    localStorage.removeItem(LEGACY_KEY);
    migrated = true;
  });
}

export function addTabDivider(after: string): string {
  const id = crypto.randomUUID();
  update({ type: "add", divider: { id, after, name: "" } });
  editingDividerId.set(id);
  return id;
}
export const removeTabDivider = (id: string) => update({ type: "remove", id });
export const renameTabDivider = (id: string, name: string) =>
  update({ type: "rename", id, name: name.trim().slice(0, MAX_NAME) });
export const moveTabDivider = (id: string, after: string) =>
  update({ type: "move", id, after });

export function reanchorPreviewDividers(closedTabId: string, after: string): void {
  saved = saved.map(item => item.after === closedTabId ? { ...item, after } : item);
  render();
}
