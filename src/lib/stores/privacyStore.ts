import { writable, derived } from "svelte/store";
import { safeStorage } from "../services/storage.js";
import { createAutoSkipPreference } from "../services/auto-skip-preference.js";
import { setNativeAutoSkipAds } from "../services/bridge/privacy";
import { describeError } from "../services/user-errors.js";
import type { PrivacyStats } from "../types/browser";
import {
  fetchPrivacyStats,
  toggleAdblockShield,
  isTauriRuntime,
} from "../services/tauri-bridge";

const confirmedPrivacyStats = writable<PrivacyStats>({
  shield_enabled: true,
  auto_skip_ads_enabled: false,
  blocked_count: 0,
  hibernated_tabs_count: 0,
  supports_live_policy_updates: false,
});
export const privacyStats = { subscribe: confirmedPrivacyStats.subscribe };

const autoSkipBusy = writable(false);
const shieldBusy = writable(false);
export const privacyBusy = derived([autoSkipBusy, shieldBusy], values => values.some(Boolean));
export const privacyError = writable("");
const autoSkip = createAutoSkipPreference({
  storage: safeStorage,
  apply: async (enabled: boolean) => {
    const stats = await setNativeAutoSkipAds(enabled);
    confirmedPrivacyStats.set(stats);
    return stats.auto_skip_ads_enabled;
  },
  changed: (value: boolean, busy: boolean) => {
    confirmedPrivacyStats.update(stats => ({ ...stats, auto_skip_ads_enabled: value }));
    autoSkipBusy.set(busy);
  },
});

export async function setAutoSkipAds(enabled: boolean): Promise<void> {
  privacyError.set("");
  try { await autoSkip.set(enabled); }
  catch (error) { privacyError.set(describeError(error)); throw error; }
}

export async function initializePrivacyStats(): Promise<void> {
  const revision = privacyRevision;
  const initial = await fetchPrivacyStats();
  if (privacyRevision === revision) applyPrivacyStats(initial);
  try { await autoSkip.initialize(); }
  catch (error) { privacyError.set(describeError(error)); }
}

export async function flipAdblockShield(): Promise<void> {
  shieldBusy.set(true);
  privacyError.set("");
  try {
    if (!isTauriRuntime()) {
      confirmedPrivacyStats.update((s) => ({ ...s, shield_enabled: !s.shield_enabled }));
      return;
    }
    const updated = await toggleAdblockShield();
    confirmedPrivacyStats.set(updated);
  } catch (error) { privacyError.set(describeError(error)); throw error; }
  finally { shieldBusy.set(false); }
}


let privacyRevision = 0;
export function applyPrivacyStats(stats: PrivacyStats): void {
  privacyRevision++;
  confirmedPrivacyStats.set(stats);
}
