import { createSharedLifecycle } from "../services/lifecycle.js";
import { get, writable } from "svelte/store";
import { fetchRuntimeStatus, onRuntimeStatus, requestRuntimeRestart, unavailableRuntime } from "../services/bridge/webview-runtime";
import { applyShutdownState } from "./shutdownStore";
import { describeError } from "../services/user-errors.js";
import type { WebviewRuntimeStatus } from "../types/webview-runtime";

export const runtimeStatus = writable(unavailableRuntime);
export const runtimeBusy = writable(false);
export const runtimeError = writable("");
export const dismissedRuntimeVersion = writable("");
function apply(status: WebviewRuntimeStatus) { if (status.revision >= get(runtimeStatus).revision) runtimeStatus.set(status); }
function startRuntime(): () => void {
  let disposed = false, stop: (() => void) | undefined;
  void (async () => {
    stop = await onRuntimeStatus(value => { if (!disposed) apply(value); });
    if (disposed) { stop(); return; }
    const status = await fetchRuntimeStatus();
    if (!disposed) apply(status);
  })().catch(error => { if (!disposed) runtimeError.set(describeError(error)); });
  return () => { disposed = true; stop?.(); };
}
export async function restartRuntime() {
  if (get(runtimeBusy) || !get(runtimeStatus).restartRequired) return;
  runtimeBusy.set(true); runtimeError.set("");
  try { applyShutdownState(await requestRuntimeRestart()); }
  catch (error) { runtimeError.set(describeError(error)); }
  finally { runtimeBusy.set(false); }
}

export const initializeRuntime = createSharedLifecycle(scope => scope.own(startRuntime()));
