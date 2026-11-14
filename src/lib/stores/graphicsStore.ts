import { get, writable } from "svelte/store";
import { fetchGraphicsStatus, previewGraphics, requestGraphicsRestart, saveHardwareAcceleration } from "../services/bridge/graphics";
import { describeError } from "../services/user-errors.js";
import { applyShutdownState } from "./shutdownStore";

/** Hardware acceleration is device-specific and read before any window exists, so it is not part of backups. */
export const graphicsStatus = writable(previewGraphics);
export const graphicsBusy = writable(false);
export const graphicsError = writable("");

async function run(work: () => Promise<void>): Promise<void> {
  if (get(graphicsBusy)) return;
  graphicsBusy.set(true);
  graphicsError.set("");
  try {
    await work();
  } catch (error) {
    graphicsError.set(describeError(error));
  } finally {
    graphicsBusy.set(false);
  }
}

export const refreshGraphics = () => run(async () => graphicsStatus.set(await fetchGraphicsStatus()));
export const setHardwareAcceleration = (enabled: boolean) => run(async () => graphicsStatus.set(await saveHardwareAcceleration(enabled)));
export const restartForGraphics = () =>
  run(async () => {
    if (!get(graphicsStatus).restartRequired) return;
    applyShutdownState(await requestGraphicsRestart());
  });
