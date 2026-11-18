import { get } from "svelte/store";
import { tabs, suspendTab } from "./browserStore";
import { tabsMemory, refreshTabsMemory } from "./tabMemoryStore";
import { resourceLimits, applyNativeLimits } from "./resourceLimitsStore";
import { hibernationCandidates, usedMegabytes, isOverLimit, backgroundTabIds } from "../services/resource-limits";
import { invoke, isTauriRuntime } from "../services/bridge/runtime";
import { logIgnored } from "../services/ignored-error";

const CHECK_EVERY_MS = 6000;
/** How long a tab the native side refused to hibernate (a call, a form, a download...) is left alone before asking again. */
const REFUSED_FOR_MS = 60_000;
const refused = new Map<string, number>();
/** The memory level last sent for each tab, so only the tabs that changed are sent again. */
const levels = new Map<string, "low" | "normal">();
let checking = false;

/**
 * One check of the memory limit. The tab the user is on is never touched. The others are put to sleep, the
 * ones used longest ago first, as many as it takes to get under the limit (not one per check). The ones that
 * cannot sleep (a call, music, a form being edited, a download, pinned tabs) stay awake but are asked to use
 * the least memory they can while they are in the background, and go back to normal when they are shown.
 * Returns whether any tab was put to sleep.
 */
export async function enforceMemoryLimit(): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  const limit = get(resourceLimits).ramMb;
  if (!limit) { await restoreMemoryLevels(); return false; }
  if (checking) return false;
  checking = true;
  let slept = false;
  try {
    await refreshTabsMemory(true);
    const measured = get(tabsMemory);
    let used = usedMegabytes(measured);
    const now = Date.now();
    for (const tab of hibernationCandidates(get(tabs))) {
      if (!isOverLimit(used, limit)) break;
      if ((refused.get(tab.id) ?? 0) > now) continue;
      try {
        await suspendTab(tab.id);
        used -= usedMegabytes({ [tab.id]: measured[tab.id] ?? 0 });
        refused.delete(tab.id);
        slept = true;
      } catch {
        // Expected: the native side protects tabs with something going on. Not an error; ask again later.
        refused.set(tab.id, now + REFUSED_FOR_MS);
      }
    }
    await applyMemoryLevels(true);
    return slept;
  } finally {
    checking = false;
  }
}

/** Background tabs that are still open get the low memory level while a limit is on; the visible ones the normal level. */
async function applyMemoryLevels(limited: boolean): Promise<void> {
  const open = get(tabs).filter(tab => !tab.is_hibernated);
  const background = limited ? backgroundTabIds(open) : new Set<string>();
  const low: string[] = [];
  const normal: string[] = [];
  for (const tab of open) {
    const want = background.has(tab.id) ? "low" : "normal";
    if ((levels.get(tab.id) ?? "normal") === want) continue;
    (want === "low" ? low : normal).push(tab.id);
  }
  const alive = new Set(open.map(tab => tab.id));
  for (const id of [...levels.keys()]) if (!alive.has(id)) levels.delete(id);
  if (!low.length && !normal.length) return;
  try {
    await invoke<void>("set_tab_memory_levels", { low, normal });
    for (const id of low) levels.set(id, "low");
    for (const id of normal) levels.set(id, "normal");
  } catch (error) { logIgnored(error); }
}

/** With no limit every tab goes back to the normal memory level. */
async function restoreMemoryLevels(): Promise<void> {
  if ([...levels.values()].every(level => level === "normal")) return;
  await applyMemoryLevels(false);
}

/** Starts the limits: sends the processor and network limits to the native side and watches the memory. */
export function startResourceLimits(): () => void {
  void applyNativeLimits().catch(logIgnored);
  const timer = setInterval(() => void enforceMemoryLimit().catch(logIgnored), CHECK_EVERY_MS);
  // Changing the limit or switching tabs is checked right away, so a tab that is shown stops being held back
  // and a new limit does not wait for the next check.
  let soon: ReturnType<typeof setTimeout> | undefined;
  const later = () => { clearTimeout(soon); soon = setTimeout(() => void enforceMemoryLimit().catch(logIgnored), 400); };
  let lastLimit = get(resourceLimits).ramMb;
  const stopLimit = resourceLimits.subscribe(value => { if (value.ramMb !== lastLimit) { lastLimit = value.ramMb; later(); } });
  let lastActive = "";
  const stopTabs = tabs.subscribe(list => {
    const active = list.filter(tab => tab.is_active || tab.is_visible).map(tab => tab.id).sort().join(",");
    if (active !== lastActive) { lastActive = active; if (get(resourceLimits).ramMb) later(); }
  });
  return () => { clearInterval(timer); clearTimeout(soon); stopLimit(); stopTabs(); };
}
