import { get, writable } from "svelte/store";
import { tabs } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { mediaState, mediaError, pipBusy, refreshMedia, enterVideo, exitVideo } from "./mediaStore";
import { closeMedia } from "./mediaPopoverStore";
import { createPipController } from "../services/pip-controller.js";
import { isTauriRuntime } from "../services/bridge/runtime";
import { describeError } from "../services/user-errors.js";
import {initializePipAppearance} from "./pipAppearanceStore";

export const pipButtonBusy = writable(false);
export const toggleVideoPip = createPipController({
  readContext() {
    const active = get(tabs).find(tab => tab.is_active);
    if (!active) return null;
    return { tabId: active.id, url: active.url, workspaceId: get(activeWorkspaceId),
      documentKey: get(mediaState).tabs.find(tab => tab.tab_id === active.id)?.document_key ?? "" };
  },
  hasPip: () => get(mediaState).tabs.some(tab => tab.frames.some(frame => frame.picture_in_picture)),
  isBusy: () => !isTauriRuntime() || get(pipBusy),
  async refresh(id) { await initializePipAppearance();return (await refreshMedia(id, true))?.tabs.find(tab => tab.tab_id === id); },
  closeVolume: closeMedia,
  enter: enterVideo,
  exit: exitVideo,
  onBusy(busy) { pipButtonBusy.set(busy); if (busy) mediaError.set(""); },
  onError: error => mediaError.set(describeError(error)),
});
