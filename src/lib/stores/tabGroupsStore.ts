import { get, writable } from "svelte/store";
import { logIgnored } from "../services/ignored-error";
import { isTauriRuntime } from "../services/bridge/runtime";
import { fetchTabGroups, saveTabGroups } from "../services/bridge/tab-groups";

export const GROUP_COLORS = ["grey", "blue", "red", "yellow", "green", "pink", "purple", "cyan"] as const;
export type GroupColor = typeof GROUP_COLORS[number];
/** The shade each group color is drawn with, in the sidebar and in the side list. */
export const GROUP_COLOR_HEX: Record<GroupColor, string> = {
  grey: "#9aa0a6", blue: "#5b9cf5", red: "#f26b6b", yellow: "#f2c14e", green: "#4cc38a", pink: "#f472b6", purple: "#a78bfa", cyan: "#4cc9f0",
};
export type GroupMode = "collapse" | "popup";
export interface TabGroup { id: string; name: string; color: GroupColor; mode: GroupMode; collapsed: boolean }
export interface TabGroups { groups: TabGroup[]; members: Record<string, string> }
const MAX_NAME = 40;

export const tabGroups = writable<TabGroups>({ groups: [], members: {} });
/** The group whose name is being typed right now. */
export const editingGroupId = writable<string | null>(null);

// Groups live in the app database. Nothing is written until the saved groups have been read, so an
// early change can never overwrite what was saved.
let loaded = false;
let timer: ReturnType<typeof setTimeout> | undefined;
function flush(): void {
  timer = undefined;
  if (!isTauriRuntime()) return;
  saveTabGroups(get(tabGroups)).catch(logIgnored);
}
tabGroups.subscribe(() => {
  if (!loaded) return;
  clearTimeout(timer);
  timer = setTimeout(flush, 150);
});

export async function loadTabGroups(): Promise<void> {
  if (loaded) return;
  let saved: TabGroups = { groups: [], members: {} };
  try { if (isTauriRuntime()) saved = await fetchTabGroups(); }
  catch (error) { logIgnored(error); return; } // Stay unloaded: nothing is saved over what could not be read.
  const pending = get(tabGroups);
  const known = new Set(saved.groups.map(group => group.id));
  const extra = pending.groups.filter(group => !known.has(group.id));
  tabGroups.set({
    groups: [...saved.groups, ...extra],
    members: { ...saved.members, ...Object.fromEntries(Object.entries(pending.members).filter(([, group]) => extra.some(item => item.id === group))) },
  });
  loaded = true;
}

/** Drops groups that have no tab left. */
function tidy(state: TabGroups): TabGroups {
  const used = new Set(Object.values(state.members));
  return state.groups.every(group => used.has(group.id)) ? state : { ...state, groups: state.groups.filter(group => used.has(group.id)) };
}
const change = (edit: (state: TabGroups) => TabGroups) => tabGroups.update(state => tidy(edit(state)));
const withMembers = (state: TabGroups, ids: string[], groupId: string | null): Record<string, string> => {
  const members = { ...state.members };
  for (const id of ids) { if (groupId) members[id] = groupId; else delete members[id]; }
  return members;
};
const update = (id: string, patch: Partial<TabGroup>) =>
  change(state => ({ ...state, groups: state.groups.map(group => group.id === id ? { ...group, ...patch } : group) }));

export function createGroup(tabIds: string[]): string {
  const id = crypto.randomUUID();
  change(state => ({
    groups: [...state.groups, { id, name: "", color: GROUP_COLORS[(state.groups.length + 1) % GROUP_COLORS.length], mode: "collapse", collapsed: false }],
    members: withMembers(state, tabIds, id),
  }));
  editingGroupId.set(id);
  return id;
}
export const addTabsToGroup = (tabIds: string[], groupId: string) =>
  change(state => state.groups.some(group => group.id === groupId) ? { ...state, members: withMembers(state, tabIds, groupId) } : state);
export const removeTabsFromGroup = (tabIds: string[]) =>
  change(state => tabIds.some(id => id in state.members) ? { ...state, members: withMembers(state, tabIds, null) } : state);
export const renameGroup = (id: string, name: string) => update(id, { name: name.trim().slice(0, MAX_NAME) });
export const setGroupColor = (id: string, color: GroupColor) => update(id, { color });
export const setGroupMode = (id: string, mode: GroupMode) => update(id, { mode, collapsed: false });
export const toggleGroupCollapsed = (id: string) =>
  change(state => ({ ...state, groups: state.groups.map(group => group.id === id ? { ...group, collapsed: !group.collapsed } : group) }));
export const ungroup = (id: string) =>
  change(state => ({
    groups: state.groups.filter(group => group.id !== id),
    members: Object.fromEntries(Object.entries(state.members).filter(([, group]) => group !== id)),
  }));
