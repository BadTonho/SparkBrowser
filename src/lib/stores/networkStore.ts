import { get, writable } from "svelte/store";
import { fetchNetworkStatus, previewNetwork, requestNetworkRestart, saveDohProvider, type DohProvider } from "../services/bridge/network";
import { describeError } from "../services/user-errors.js";
import { applyShutdownState } from "./shutdownStore";

/** The DNS over HTTPS proxy is part of how the engine starts, so the choice is not part of backups. */
export const networkStatus = writable(previewNetwork);
export const networkBusy = writable(false);
export const networkError = writable("");

async function run(work: () => Promise<void>): Promise<void> {
  if (get(networkBusy)) return;
  networkBusy.set(true);
  networkError.set("");
  try {
    await work();
  } catch (error) {
    networkError.set(describeError(error));
  } finally {
    networkBusy.set(false);
  }
}

export const refreshNetwork = () => run(async () => networkStatus.set(await fetchNetworkStatus()));
export const setDohProvider = (provider: DohProvider | null) => run(async () => networkStatus.set(await saveDohProvider(provider)));
export const restartForNetwork = () =>
  run(async () => {
    if (!get(networkStatus).restartRequired) return;
    applyShutdownState(await requestNetworkRestart());
  });
