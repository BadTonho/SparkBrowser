import { invoke, isTauriRuntime } from "./runtime";

/** `true` only inside a private window; browser preview and normal windows are never private. */
export const fetchPrivateWindowState = () =>
  isTauriRuntime() ? invoke<boolean>("get_private_window_state") : Promise.resolve(false);
