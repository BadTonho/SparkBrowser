import { get } from "svelte/store";
import { tabs, suspendTab } from "./browserStore";
import { tabsMemory, refreshTabsMemory } from "./tabMemoryStore";
import { resourceLimits, applyNativeLimits } from "./resourceLimitsStore";
import { hibernationCandidates, usedMegabytes, isOverLimit } from "../services/resource-limits";
import { isTauriRuntime } from "../services/bridge/runtime";
import { logIgnored } from "../services/ignored-error";

const CHECK_EVERY_MS = 6000;
/** Tabs offered per check; the native side refuses the protected ones, so a few are tried. */
const TRIES = 3;
let checking = false;

/** One check: while the tabs use more than the limit, the least recently used tab is hibernated, one per check. */
export async function enforceMemoryLimit(): Promise<boolean> {
  const limit = get(resourceLimits).ramMb;
  if (!limit || checking || !isTauriRuntime()) return false;
  checking = true;
  try {
    await refreshTabsMemory(true);
    if (!isOverLimit(usedMegabytes(get(tabsMemory)), limit)) return false;
    for (const tab of hibernationCandidates(get(tabs)).slice(0, TRIES)) {
      try { await suspendTab(tab.id); return true; }
      catch (error) { logIgnored(error); }
    }
    return false;
  } finally {
    checking = false;
  }
}

/** Starts the limits: sends the processor and network limits to the native side and watches the memory. */
export function startResourceLimits(): () => void {
  void applyNativeLimits().catch(logIgnored);
  const timer = setInterval(() => void enforceMemoryLimit().catch(logIgnored), CHECK_EVERY_MS);
  return () => clearInterval(timer);
}
