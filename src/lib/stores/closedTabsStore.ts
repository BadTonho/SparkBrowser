import { get, writable } from "svelte/store";
import { isTauriRuntime } from "../services/bridge/runtime";
import { clearClosedTabs, fetchClosedTabs, onClosedTabsUpdated, reopenClosedTab } from "../services/bridge/closed-tabs";
import { describeError, UserError } from "../services/user-errors.js";
import { applyTabsSnapshot } from "./browserStore";
import type { ClosedTab } from "../types/closed-tabs";
export const closedTabs = writable<ClosedTab[]>([]);
export const closedTabsError = writable("");
export const closedTabsBusy = writable(false);
export const closedTabsNotice = writable<"missing_workspace" | "empty" | null>(null);
let revision = 0;
export async function refreshClosedTabs(): Promise<void> {
  if (!isTauriRuntime()) return;
  const request = ++revision;
  try { const rows = await fetchClosedTabs(); if (request === revision) closedTabs.set(rows); }
  catch (error) { if (request === revision) closedTabsError.set(describeError(error)); }
}
async function action(work: () => Promise<void>): Promise<void> {
  if (get(closedTabsBusy)) return;
  closedTabsBusy.set(true); closedTabsError.set(""); closedTabsNotice.set(null);
  try { if (!isTauriRuntime()) throw new UserError("desktop.required", "Recently closed tabs require the desktop application"); await work(); }
  catch (error) { closedTabsError.set(describeError(error)); }
  finally { closedTabsBusy.set(false); }
}
export const restoreClosedTab = (id: number | null = null) => action(async () => {
  const result = await reopenClosedTab(id);
  if (!result) closedTabsNotice.set("empty");
  else { applyTabsSnapshot(result.snapshot); if (result.workspace_missing) closedTabsNotice.set("missing_workspace"); }
  await refreshClosedTabs();
});
export const wipeClosedTabs = () => action(async () => { await clearClosedTabs(); await refreshClosedTabs(); });
export function dismissClosedNotice(): void { closedTabsError.set(""); closedTabsNotice.set(null); }
export function initializeClosedTabEvents(): () => void {
  let disposed = false; let stop: (() => void) | undefined;
  void onClosedTabsUpdated(rows => { if (!disposed) { ++revision; closedTabs.set(rows); } })
    .then(unlisten => { if (disposed) unlisten(); else { stop = unlisten; void refreshClosedTabs(); } })
    .catch(error => closedTabsError.set(describeError(error)));
  return () => { disposed = true; stop?.(); };
}
