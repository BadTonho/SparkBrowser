import { invoke, isTauriRuntime } from "./runtime";

export type PanelName = "quick-settings" | "tools" | "downloads" | "settings";

const requested = new Set<PanelName>();

/**
 * Asks the native side to create a panel's web view ahead of the click. Best effort and once per
 * panel per run: a failure only means the first open is as slow as it used to be.
 */
export function preparePanel(panel: PanelName): void {
  if (!isTauriRuntime() || requested.has(panel)) return;
  requested.add(panel);
  void invoke<void>("prepare_panel", { panel }).catch(() => requested.delete(panel));
}
