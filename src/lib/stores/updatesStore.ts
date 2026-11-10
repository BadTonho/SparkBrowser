import { createSharedLifecycle } from "../services/lifecycle.js";
import { tr } from "../i18n";
import { describeError } from "../services/user-errors.js";
import { get, writable } from "svelte/store";
import { changeUpdateChannel, checkForUpdates, downloadUpdate, fetchUpdateState, installUpdate, onUpdateState, previewUpdateState } from "../services/bridge/updates";
import { createUpdateGate } from "../services/update-snapshots.js";
import type { UpdateChannel, UpdateState } from "../types/updates";

export const updateState = writable(previewUpdateState);
export const updateActionError = writable("");
export const updateBusy = writable(false);
const apply = createUpdateGate((value: UpdateState) => updateState.set(value));
async function action(work: () => Promise<void>) {
  if (get(updateBusy)) return;
  updateBusy.set(true); updateActionError.set("");
  try { await work(); } catch (error) { updateActionError.set(describeError(error)); }
  finally { updateBusy.set(false); }
}
function startUpdates(): () => void {
  let disposed = false;
  let stop: (() => void) | undefined;
  void (async () => {
    stop = await onUpdateState(value => { if (!disposed) apply(value); });
    if (disposed) { stop(); return; }
    const state = await fetchUpdateState();
    if (disposed) return;
    apply(state);
    if (state.supported) {
      const checked = await checkForUpdates(true);
      if (!disposed) apply(checked);
    }
  })().catch(error => { if (!disposed) updateActionError.set(describeError(error)); });
  return () => { disposed = true; stop?.(); };
}
export const selectUpdateChannel = (channel: UpdateChannel) => action(async () => { apply(await changeUpdateChannel(channel)); });
export async function setImportedUpdateChannel(channel: UpdateChannel): Promise<void> { apply(await changeUpdateChannel(channel)); }
export const checkUpdates = () => action(async () => { apply(await checkForUpdates()); });
export const startUpdateDownload = () => action(async () => {
  const version = get(updateState).update?.version;
  if (version) apply(await downloadUpdate(version));
});
export const startUpdateInstall = () => action(async () => {
  const version = get(updateState).update?.version;
  if (!version || get(updateState).status !== "ready") return;
  if (!window.confirm(tr("common.install_confirm", { version }))) return;
  await installUpdate(version, false);
});

export const initializeUpdates = createSharedLifecycle(scope => scope.own(startUpdates()));
