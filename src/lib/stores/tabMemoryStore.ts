import { writable } from "svelte/store";
import { invoke, isTauriRuntime } from "../services/tauri-bridge";

export const tabsMemory = writable<Record<string, number>>({});

const lastKnownMemory: Record<string, number> = {};
const DEFAULT_SAVINGS_BYTES = 95 * 1024 * 1024; // ~95 MB typical WebView2 footprint

let lastFetch = 0;
let isFetching = false;

export async function refreshTabsMemory(force = false): Promise<void> {
  const now = Date.now();
  if (!force && (isFetching || now - lastFetch < 2000)) return;
  if (!isTauriRuntime()) return;

  isFetching = true;
  try {
    const memory = await invoke<Record<string, number>>("get_tabs_memory");
    if (memory) {
      for (const [id, bytes] of Object.entries(memory)) {
        if (bytes > 0) {
          lastKnownMemory[id] = bytes;
        }
      }
      tabsMemory.set(memory);
      lastFetch = Date.now();
    }
  } catch (error) {
    console.warn("Failed to refresh tabs memory:", error);
  } finally {
    isFetching = false;
  }
}

export function formatTabMemory(bytes?: number, unavailable = "—"): string {
  if (bytes == null || bytes === 0) return unavailable;
  const mb = Math.round(bytes / (1024 * 1024));
  if (mb < 1) return "< 1 MB";
  return `${mb} MB`;
}

export function getTabSavingsBytes(tabId: string): number {
  return lastKnownMemory[tabId] ?? DEFAULT_SAVINGS_BYTES;
}

export function formatTabSavings(tabId: string): string {
  const bytes = getTabSavingsBytes(tabId);
  const mb = Math.round(bytes / (1024 * 1024));
  return `~${mb} MB`;
}
