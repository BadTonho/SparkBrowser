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

export const privacyStats = writable<PrivacyStats>({
  shield_enabled: true,
  auto_skip_ads_enabled: false,
  blocked_count: 0,
  hibernated_tabs_count: 0,
  supports_live_policy_updates: false,
});

const autoSkipBusy = writable(false);
const shieldBusy = writable(false);
export const privacyBusy = derived([autoSkipBusy, shieldBusy], values => values.some(Boolean));
export const privacyError = writable("");
const autoSkip = createAutoSkipPreference({
  storage: safeStorage,
  apply: async (enabled: boolean) => {
    const stats = await setNativeAutoSkipAds(enabled);
    privacyStats.set(stats);
    return stats.auto_skip_ads_enabled;
  },
  changed: (value: boolean, busy: boolean) => {
    privacyStats.update(stats => ({ ...stats, auto_skip_ads_enabled: value }));
    autoSkipBusy.set(busy);
  },
});

export async function setAutoSkipAds(enabled: boolean): Promise<void> {
  privacyError.set("");
  try { await autoSkip.set(enabled); }
  catch (error) { privacyError.set(describeError(error)); throw error; }
}

export async function initializePrivacyStats(): Promise<void> {
  const initial = await fetchPrivacyStats();
  privacyStats.set(initial);
  try { await autoSkip.initialize(); }
  catch (error) { privacyError.set(describeError(error)); }
}

export async function flipAdblockShield(): Promise<void> {
  shieldBusy.set(true);
  privacyError.set("");
  try {
    if (!isTauriRuntime()) {
      privacyStats.update((s) => ({ ...s, shield_enabled: !s.shield_enabled }));
      return;
    }
    const updated = await toggleAdblockShield();
    privacyStats.set(updated);
  } catch (error) { privacyError.set(describeError(error)); throw error; }
  finally { shieldBusy.set(false); }
}
