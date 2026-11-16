import { createSharedLifecycle } from "../services/lifecycle.js";
import { get, writable } from "svelte/store";
import { idleShutdown, fetchShutdown, onShutdown, prepareShutdown, confirmShutdown, cancelShutdown } from "../services/bridge/shutdown";
import { setOverlayVisibility } from "../services/bridge/overlays";
import { describeError, UserError } from "../services/user-errors.js";
import { safeStorage } from "../services/storage.js";
import { reviewNeedsAttention, shutdownPresentation, SHUTDOWN_REVEAL_DELAY_MS } from "../services/shutdown-skip.js";
import { flushBackupSyncBeforeShutdown } from "./backupStore";
import { noteWriter } from "./notesStore";
import type { ShutdownState } from "../types/shutdown";

export const shutdownState = writable<ShutdownState>(idleShutdown);
export const shutdownError = writable("");
export const shutdownBusy = writable(false);
/** The dialog is shown for a decision or an error at once, and for saving only when it takes long. */
export const shutdownVisible = writable(false);
let revealTimer: ReturnType<typeof setTimeout> | undefined;
function setVisible(visible: boolean) {
  if (get(shutdownVisible) === visible) return;
  shutdownVisible.set(visible);
  void setOverlayVisibility(visible, "shutdown");
}
function present(state: ShutdownState) {
  const mode = shutdownPresentation(state, get(skipExitReview));
  if (mode !== "working") { clearTimeout(revealTimer); revealTimer = undefined; }
  if (mode === "hidden") setVisible(false);
  else if (mode === "attention") setVisible(true);
  else if (!get(shutdownVisible) && revealTimer === undefined) {
    revealTimer = setTimeout(() => {
      revealTimer = undefined;
      if (shutdownPresentation(get(shutdownState), get(skipExitReview)) === "working") setVisible(true);
    }, SHUTDOWN_REVEAL_DELAY_MS);
  }
}
const SKIP_EXIT_REVIEW_KEY = "spark_skip_exit_review";
/** "Don't show again": skips the exit review unless something would be lost for good. */
export const skipExitReview = writable(safeStorage.getItem(SKIP_EXIT_REVIEW_KEY) === "true");
export function setSkipExitReview(enabled: boolean): boolean {
  skipExitReview.set(enabled);
  return safeStorage.setItem(SKIP_EXIT_REVIEW_KEY, String(enabled));
}
let flushing: string | null = null;
function apply(state: ShutdownState) {
  if (state.revision < get(shutdownState).revision) return;
  shutdownState.set(state);
  shutdownError.set(state.error ? describeError(new UserError(state.error.split(":")[0], state.error)) : "");
  present(state);
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
    if (state.review && !reviewNeedsAttention(state, get(skipExitReview))) {
      apply(await confirmShutdown(attempt, state.review.revision));
    }
  } catch (error) { shutdownError.set(describeError(error)); }
  finally { shutdownBusy.set(false); }
}
export async function exitAnyway(dontAskAgain = false) {
  const state = get(shutdownState);
  if (!state.attempt || !state.review || get(shutdownBusy)) return;
  if (dontAskAgain && state.purpose === "exit" && !state.update_version) setSkipExitReview(true);
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
