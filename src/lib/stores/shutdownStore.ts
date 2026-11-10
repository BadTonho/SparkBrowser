import { createSharedLifecycle } from "../services/lifecycle.js";
import { get, writable } from "svelte/store";
import { idleShutdown, fetchShutdown, onShutdown, prepareShutdown, confirmShutdown, cancelShutdown } from "../services/bridge/shutdown";
import { setOverlayVisibility } from "../services/bridge/overlays";
import { describeError, UserError } from "../services/user-errors.js";
import { flushBackupSyncBeforeShutdown } from "./backupStore";
import { noteWriter } from "./notesStore";
import type { ShutdownState } from "../types/shutdown";

export const shutdownState = writable<ShutdownState>(idleShutdown);
export const shutdownError = writable("");
export const shutdownBusy = writable(false);
let flushing: string | null = null;
function apply(state: ShutdownState) {
  if (state.revision < get(shutdownState).revision) return;
  shutdownState.set(state);
  shutdownError.set(state.error ? describeError(new UserError(state.error.split(":")[0], state.error)) : "");
  void setOverlayVisibility(state.phase !== "idle", "shutdown");
  if (state.phase === "flushing" && state.attempt && flushing !== state.attempt) {
    flushing = state.attempt;
    void retryShutdown();
  }
  if (state.phase === "idle") flushing = null;
}
export { apply as applyShutdownState };
export async function retryShutdown() {
  const attempt = get(shutdownState).attempt;
  if (!attempt || get(shutdownBusy)) return;
  shutdownBusy.set(true); shutdownError.set("");
  try {
    await noteWriter.flushAll();
    await flushBackupSyncBeforeShutdown();
    const state = await prepareShutdown(attempt);
    apply(state);
    if (state.review && !state.review.tabs.length && !state.review.downloads.length && !state.review.native_downloads.length) {
      apply(await confirmShutdown(attempt, state.review.revision));
    }
  } catch (error) { shutdownError.set(describeError(error)); }
  finally { shutdownBusy.set(false); }
}
export async function exitAnyway() {
  const state = get(shutdownState);
  if (!state.attempt || !state.review || get(shutdownBusy)) return;
  shutdownBusy.set(true); shutdownError.set("");
  try { apply(await confirmShutdown(state.attempt, state.review.revision)); }
  catch (error) { shutdownError.set(describeError(error)); }
  finally { shutdownBusy.set(false); }
}
export async function continueBrowsing() {
  const attempt = get(shutdownState).attempt;
  if (!attempt || get(shutdownBusy)) return;
  try { apply(await cancelShutdown(attempt)); }
  catch (error) { shutdownError.set(describeError(error)); }
}
function startShutdown() {
  let disposed = false;
  let stop: (() => void) | undefined;
  void (async () => {
    stop = await onShutdown(state => { if (!disposed) apply(state); });
    if (disposed) { stop(); return; }
    const state = await fetchShutdown();
    if (!disposed) apply(state);
  })().catch(error => shutdownError.set(describeError(error)));
  return () => { disposed = true; stop?.(); };
}

export const initializeShutdown = createSharedLifecycle(scope => scope.own(startShutdown()));
