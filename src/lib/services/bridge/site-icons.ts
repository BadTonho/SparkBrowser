import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";

/** Cached icons (as `data:` URLs) for the given page addresses; pages with no cached icon are absent. */
export const fetchSiteIcons = (urls: string[]): Promise<Record<string, string>> =>
  isTauriRuntime() ? invoke<Record<string, string>>("site_icons", { urls }) : Promise.resolve({});

export const onSiteIconsChanged = (callback: () => void) =>
  isTauriRuntime() ? listen("site-icons-changed", () => callback()) : Promise.resolve(() => {});
