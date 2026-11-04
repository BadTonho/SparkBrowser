import { writable } from "svelte/store";
import { invoke, isTauriRuntime } from "../services/tauri-bridge";

export const tabsMemory = writable<Record<string, number>>({});

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
      tabsMemory.set(memory);
      lastFetch = Date.now();
    }
  } catch (error) {
    console.warn("Failed to refresh tabs memory:", error);
  } finally {
    isFetching = false;
  }
}

export function formatTabMemory(bytes?: number): string {
  if (bytes == null || bytes === 0) return "< 1 MB";
  const mb = Math.round(bytes / (1024 * 1024));
  if (mb < 1) return "< 1 MB";
  return `${mb} MB`;
}
