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
import { logIgnored } from "../services/ignored-error";

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
  // Video discovery still needs polling. An idle focused tab uses a slower interval;
  // a newly added video on such a tab is found within POLL_WITHOUT_MEDIA_MS.
  const POLL_WITH_MEDIA_MS = 1000, POLL_WITHOUT_MEDIA_MS = 3000;
  let pollTimer: ReturnType<typeof setTimeout> | undefined;
  function activeTabHasMedia() {
    const active = get(tabs).find(tab => tab.is_active);
    const media = get(mediaState).tabs.find(tab => tab.tab_id === active?.id);
    return Boolean(media && (media.audible || media.mute_available || media.frames.some(frame => frame.playing || frame.picture_in_picture || frame.videos.length > 0)));
  }
  function schedulePoll() {
    if (disposed) return;
    pollTimer = setTimeout(() => { refreshFocused(); schedulePoll(); }, activeTabHasMedia() ? POLL_WITH_MEDIA_MS : POLL_WITHOUT_MEDIA_MS);
  }
  schedulePoll();
  document.addEventListener("visibilitychange", refreshFocused);

  void getCurrentWindow().onFocusChanged(event => {
    focused = event.payload;
    if (focused) refreshFocused();
  }).then(stop => { if (disposed) stop(); else stopFocus = stop; }).catch(logIgnored);
  void (async () => {
    stopMedia = await onTabMedia(snapshot => { if (!disposed) applyMedia(snapshot); });
    if (disposed) { stopMedia(); return; }
    const snapshot = await fetchTabMedia();
    if (!disposed) applyMedia(snapshot);
  })().catch(error => { if (!disposed) mediaError.set(describeError(error)); });

  return () => {
    disposed = true;
    clearTimeout(pollTimer);
    stopTabs(); stopState(); stopMedia?.(); stopFocus?.();
    stopPip();
    document.removeEventListener("visibilitychange", refreshFocused);
  };
}

export const initializeMedia = createSharedLifecycle(scope => scope.own(startMedia()));
