import { createSharedLifecycle } from "../services/lifecycle.js";
import { get } from "svelte/store";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { tabs } from "./browserStore";
import { mediaState, applyMedia, mediaError, refreshMedia } from "./mediaStore";
import { fetchTabMedia, onTabMedia } from "../services/bridge/media";
import { isTauriRuntime } from "../services/bridge/runtime";
import { describeError } from "../services/user-errors.js";
import { initializePipControls } from "./pipLifecycle";
import {initializePipAppearance} from "./pipAppearanceStore";

/** One focused-tab report loop serves volume and PiP, including dynamically added videos. */
function startMedia() {
  if (!isTauriRuntime()) return () => {};
  let disposed = false;
  let focused = true;
  let lastTarget = "";
  let stopMedia: (() => void) | undefined;
  let stopFocus: (() => void) | undefined;
  const stopPip = initializePipControls();
  void initializePipAppearance().catch(error=>{if(!disposed)mediaError.set(describeError(error));});

  function refreshFocused() {
    const active = get(tabs).find(tab => tab.is_active);
    if (disposed || !focused || document.hidden || !active || active.is_hibernated || active.navigation?.loading) return;
    void refreshMedia(active.id, true);
  }
  function reconcile() {
    const active = get(tabs).find(tab => tab.is_active);
    const available = get(mediaState).tabs.find(tab => tab.tab_id === active?.id)?.mute_available ?? false;
    // Media report revisions and playback signals must not trigger another report.
    const target = JSON.stringify([active?.id, active?.url, active?.is_hibernated, active?.navigation?.loading, available]);
    if (target === lastTarget) return;
    lastTarget = target;
    refreshFocused();
  }
  const stopTabs = tabs.subscribe(reconcile);
  const stopState = mediaState.subscribe(reconcile);
  const timer = setInterval(refreshFocused, 1000);
  document.addEventListener("visibilitychange", refreshFocused);

  void getCurrentWindow().onFocusChanged(event => {
    focused = event.payload;
    if (focused) refreshFocused();
  }).then(stop => { if (disposed) stop(); else stopFocus = stop; }).catch(() => {});
  void (async () => {
    stopMedia = await onTabMedia(snapshot => { if (!disposed) applyMedia(snapshot); });
    if (disposed) { stopMedia(); return; }
    const snapshot = await fetchTabMedia();
    if (!disposed) applyMedia(snapshot);
  })().catch(error => { if (!disposed) mediaError.set(describeError(error)); });

  return () => {
    disposed = true;
    clearInterval(timer);
    stopTabs(); stopState(); stopMedia?.(); stopFocus?.();
    stopPip();
    document.removeEventListener("visibilitychange", refreshFocused);
  };
}

export const initializeMedia = createSharedLifecycle(scope => scope.own(startMedia()));
