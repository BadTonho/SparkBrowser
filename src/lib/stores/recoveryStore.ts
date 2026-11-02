import { writable } from "svelte/store";
import { isTauriRuntime } from "../services/bridge/runtime";
import { fetchRecoveryStatus } from "../services/bridge/recovery";
import { describeError } from "../services/user-errors.js";
import type { RecoveryStatus } from "../types/recovery";

export const recoveryStatus = writable<RecoveryStatus | null>(null);
export const recoveryError = writable("");
export const recoveryDismissed = writable(false);
export async function initializeRecovery(): Promise<void> {
  if (!isTauriRuntime()) return;
  try { recoveryStatus.set(await fetchRecoveryStatus()); }
  catch (error) { recoveryError.set(describeError(error)); }
}
export function dismissRecovery(): void { recoveryDismissed.set(true); recoveryError.set(""); }
