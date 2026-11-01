import { writable } from "svelte/store";
import type { PrivacyStats } from "../types/browser";
import {
  fetchPrivacyStats,
  toggleAdblockShield,
  isTauriRuntime,
} from "../services/tauri-bridge";

export const privacyStats = writable<PrivacyStats>({
  shield_enabled: true,
  blocked_count: 0,
  hibernated_tabs_count: 0,
});

export async function initializePrivacyStats(): Promise<void> {
  const initial = await fetchPrivacyStats();
  privacyStats.set(initial);
}

export async function flipAdblockShield(): Promise<void> {
  if (!isTauriRuntime()) {
    privacyStats.update((s) => ({ ...s, shield_enabled: !s.shield_enabled }));
    return;
  }
  const updated = await toggleAdblockShield();
  privacyStats.set(updated);
}

