import { invoke, isTauriRuntime } from "./runtime";

/** Tells the native side whether light pages should be darkened. It is held in memory there, so it is sent again at start-up. */
export const setForcedDarkMode = (enabled: boolean): Promise<void> =>
  isTauriRuntime() ? invoke<void>("set_forced_dark_mode", { enabled }) : Promise.resolve();
