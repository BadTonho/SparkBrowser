import { get, writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";

export const DEFAULT_TITLEBAR_ACTIONS = ["tools", "bookmarks", "commands", "history", "downloads", "privacy", "extensions", "settings", "zen", "qr", "whatsapp"] as const;
export type TitlebarAction = typeof DEFAULT_TITLEBAR_ACTIONS[number];
const STORAGE_KEY = "veloce_titlebar_action_order";

export function normalizeActionOrder(value: unknown): TitlebarAction[] {
  const valid = Array.isArray(value) ? value.filter((id): id is TitlebarAction =>
    DEFAULT_TITLEBAR_ACTIONS.includes(id as TitlebarAction)) : [];
  return [...new Set([...valid, ...DEFAULT_TITLEBAR_ACTIONS])];
}

function loadOrder(): TitlebarAction[] {
  try { return normalizeActionOrder(JSON.parse(safeStorage.getItem(STORAGE_KEY) ?? "null")); }
  catch { return [...DEFAULT_TITLEBAR_ACTIONS]; }
}

export const titlebarActionOrder = writable<TitlebarAction[]>(loadOrder());

/** Reorder visible slots only, preserving the saved slot of a hidden action. */
export function moveTitlebarAction(id: TitlebarAction, before: TitlebarAction | null, visible: readonly TitlebarAction[]): void {
  const order = get(titlebarActionOrder);
  if (!visible.includes(id) || before === id || (before !== null && !visible.includes(before))) return;
  const nextVisible = order.filter(action => visible.includes(action) && action !== id);
  nextVisible.splice(before === null ? nextVisible.length : nextVisible.indexOf(before), 0, id);
  let index = 0;
  const next = order.map(action => visible.includes(action) ? nextVisible[index++] : action);
  if (next.every((action, i) => action === order[i])) return;
  titlebarActionOrder.set(next);
  safeStorage.setItem(STORAGE_KEY, JSON.stringify(next));
}
