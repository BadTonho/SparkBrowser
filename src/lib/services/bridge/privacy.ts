import { invoke } from "./runtime";

import type { PrivacyStats } from "../../types/browser";

import { isTauriRuntime } from "./runtime";

export async function fetchPrivacyStats(): Promise<PrivacyStats> {
  if(!isTauriRuntime()) {
    return {
      shield_enabled: true,
      blocked_count: 0,
      hibernated_tabs_count: 0, supports_live_policy_updates: false,
    };
  }
  return await invoke<PrivacyStats>("get_privacy_stats");
}

export async function toggleAdblockShield(): Promise<PrivacyStats> {
  if(!isTauriRuntime()) {
    return {
      shield_enabled: true,
      blocked_count: 0,
      hibernated_tabs_count: 0, supports_live_policy_updates: false,
    };
  }
  return await invoke<PrivacyStats>("toggle_adblock_shield");
}
