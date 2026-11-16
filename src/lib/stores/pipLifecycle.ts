import { createSharedLifecycle } from "../services/lifecycle.js";
import { get } from "svelte/store";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { selectTab } from "./browserStore";
import { mediaState, mediaError, queueVolume, flushVolume, exitVideo, pipBusy } from "./mediaStore";
import { onPipAction, onPipFallback } from "../services/bridge/media";
import { describeError, UserError } from "../services/user-errors.js";
import { logIgnored } from "../services/ignored-error";

/** The auxiliary player can address only its current document and video session. */
function startPipControls() {
  let disposed = false;
  const stops: (() => void)[] = [];
  function register(promise: Promise<() => void>) {
    void promise.then(stop => { if (disposed) stop(); else stops.push(stop); }).catch(logIgnored);
  }
  register(onPipFallback(() => {
    mediaError.set(describeError(new UserError("media.pip_native_fallback", "Local video capture unavailable")));
  }));
  register(onPipAction(event => {
    const owner = get(mediaState).tabs.find(tab => tab.tab_id === event.tabId);
    if (!owner || !owner.frames.some(frame => frame.pip_session === event.session)) return;
    if (event.value !== null && owner.volume_status !== "unavailable") {
      queueVolume(owner.tab_id, owner.document_key, event.value, { session: event.session, sequence: event.sequence });
    }
    if (event.action === "flush") void flushVolume();
    if (event.action === "close" || event.action === "capture-failed") void (async () => {
      await flushVolume();
      // A user can close the window before the enter command has returned.
      if (get(pipBusy)) await new Promise<void>(resolve => {
        let stop: () => void = () => {};
        stop = pipBusy.subscribe(busy => { if (!busy) { queueMicrotask(() => stop()); resolve(); } });
      });
      if (get(mediaState).tabs.some(tab => tab.frames.some(frame => frame.pip_session === event.session))) {
        await exitVideo();
        if (event.action === "capture-failed") mediaError.set(describeError(new UserError("media.pip_failed", "Local video capture stopped")));
      }
    })();
    if (event.action === "return") {
      void selectTab(owner.tab_id).then(() => getCurrentWindow().setFocus()).catch(error => mediaError.set(describeError(error)));
    }
  }));
  return () => { disposed = true; stops.forEach(stop => stop()); };
}

export const initializePipControls = createSharedLifecycle(scope => scope.own(startPipControls()));
