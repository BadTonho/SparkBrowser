import { get, writable } from "svelte/store";
import { changeUpdateChannel, checkForUpdates, downloadUpdate, fetchUpdateState, installUpdate, onUpdateState, previewUpdateState } from "../services/bridge/updates";
import { createUpdateGate } from "../services/update-snapshots.js";
import { noteWriter } from "./notesStore";
import { setOverlayVisibility } from "../services/bridge/overlays";
import { invoke } from "../services/bridge/runtime";
import type { TabSnapshot } from "../types/browser";
import type { UpdateChannel, UpdateState } from "../types/updates";

export const updateState = writable(previewUpdateState);
export const updateActionError = writable("");
export const updateBusy = writable(false);
const apply = createUpdateGate((value: UpdateState) => updateState.set(value));
async function action(work: () => Promise<void>) {
  if (get(updateBusy)) return;
  updateBusy.set(true); updateActionError.set("");
  try { await work(); } catch (error) { updateActionError.set(String(error)); }
  finally { updateBusy.set(false); }
}
export function initializeUpdates(): () => void {
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
  })().catch(error => { if (!disposed) updateActionError.set(String(error)); });
  return () => { disposed = true; stop?.(); };
}
export const selectUpdateChannel = (channel: UpdateChannel) => action(async () => { apply(await changeUpdateChannel(channel)); });
export const checkUpdates = () => action(async () => { apply(await checkForUpdates()); });
export const startUpdateDownload = () => action(async () => {
  const version = get(updateState).update?.version;
  if (version) apply(await downloadUpdate(version));
});
export const startUpdateInstall = () => action(async () => {
  const version = get(updateState).update?.version;
  if (!version || get(updateState).status !== "ready") return;
  const snapshot = await invoke<TabSnapshot>("get_tabs");
  const activity = (Array.isArray(snapshot) ? snapshot : snapshot.tabs).some(tab => tab.activity?.dirty_form || tab.activity?.playing_media);
  if (!window.confirm(`Install version ${version} and restart Veloce Browser?`)) return;
  if (activity && !window.confirm("Some pages have edited forms or playing media. This page state will be lost on restart. Continue?")) return;
  await setOverlayVisibility(true, "update-install");
  try {
    await noteWriter.flushAll();
    await installUpdate(version, activity);
  } finally { await setOverlayVisibility(false, "update-install"); }
});
