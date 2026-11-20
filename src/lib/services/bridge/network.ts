import { invoke, isTauriRuntime } from "./runtime";
import type { ShutdownState } from "../../types/shutdown";

export type DohProvider = "cloudflare" | "google";
export interface NetworkStatus {
  /** What the setting says; it takes effect on the next start. */
  dohProvider: DohProvider | null;
  /** What the running process started with. */
  activeProvider: DohProvider | null;
  restartRequired: boolean;
}

/** The browser preview has no engine to route: DNS over HTTPS is off and nothing is pending. */
export const previewNetwork: NetworkStatus = { dohProvider: null, activeProvider: null, restartRequired: false };
export const fetchNetworkStatus = () => (isTauriRuntime() ? invoke<NetworkStatus>("get_network_status") : Promise.resolve(previewNetwork));
export const saveDohProvider = (provider: DohProvider | null) => invoke<NetworkStatus>("set_doh_provider", { provider });
export const requestNetworkRestart = () => invoke<ShutdownState>("restart_for_network");
