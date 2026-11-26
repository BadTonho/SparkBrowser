import { get, writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";

export const DEFAULT_TITLEBAR_ACTIONS = ["tools", "bookmarks", "commands", "history", "zen", "qr", "video", "whatsapp", "resources", "extensions", "privacy", "downloads", "settings"] as const;
export type TitlebarAction = typeof DEFAULT_TITLEBAR_ACTIONS[number];
const STORAGE_KEY = "spark_titlebar_action_order_v2";
const SHOWN_KEY = "spark_titlebar_actions_shown";

/** The buttons that come with the browser. Every other button is off until the user turns it on. */
export const DEFAULT_SHOWN_ACTIONS: readonly TitlebarAction[] = ["downloads", "privacy", "extensions", "settings"];
/** The settings button opens the menu that turns the others on, so it can never be hidden. */
export const ALWAYS_SHOWN_ACTIONS: readonly TitlebarAction[] = ["settings"];

export function normalizeShownActions(value: unknown): TitlebarAction[] {
  if (!Array.isArray(value)) return [...DEFAULT_SHOWN_ACTIONS];
  const valid = value.filter((id): id is TitlebarAction => DEFAULT_TITLEBAR_ACTIONS.includes(id as TitlebarAction));
  return [...new Set([...valid, ...ALWAYS_SHOWN_ACTIONS])];
}

function loadShown(): TitlebarAction[] {
  try { return normalizeShownActions(JSON.parse(safeStorage.getItem(SHOWN_KEY) ?? "null")); }
  catch { return [...DEFAULT_SHOWN_ACTIONS]; }
}

export const shownTitlebarActions = writable<TitlebarAction[]>(loadShown());

/** Shows or hides one toolbar button. The saved order is untouched, so a button returns to its slot. */
export function setTitlebarActionShown(id: TitlebarAction, shown: boolean): void {
  if (!DEFAULT_TITLEBAR_ACTIONS.includes(id) || (!shown && ALWAYS_SHOWN_ACTIONS.includes(id))) return;
  const current = get(shownTitlebarActions);
  if (current.includes(id) === shown) return;
  const next = shown ? [...current, id] : current.filter(action => action !== id);
  shownTitlebarActions.set(next);
  safeStorage.setItem(SHOWN_KEY, JSON.stringify(next));
}

export function normalizeActionOrder(value: unknown): TitlebarAction[] {
  const valid = Array.isArray(value) ? value.filter((id): id is TitlebarAction =>
    DEFAULT_TITLEBAR_ACTIONS.includes(id as TitlebarAction)) : [];
  const order = [...new Set(valid)];
  const missing = DEFAULT_TITLEBAR_ACTIONS.filter(id => !order.includes(id));
  // A button the saved order does not know yet goes to the left of the standard ones, never after Settings.
  const standard = order.findIndex(id => DEFAULT_SHOWN_ACTIONS.includes(id));
  order.splice(standard < 0 ? order.length : standard, 0, ...missing.filter(id => !DEFAULT_SHOWN_ACTIONS.includes(id)));
  return [...order, ...missing.filter(id => DEFAULT_SHOWN_ACTIONS.includes(id))];
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
