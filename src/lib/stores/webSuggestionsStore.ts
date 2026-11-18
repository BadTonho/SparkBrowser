import { writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";

const KEY = "spark_web_suggestions";

function initial(): boolean {
  try {
    return safeStorage?.getItem(KEY) === "true";
  } catch {
    return false;
  }
}

/** Off by default: turning it on sends what is typed in the address bar to the search engine while typing. */
export const webSuggestionsEnabled = writable<boolean>(initial());

export function setWebSuggestionsEnabled(enabled: boolean): void {
  webSuggestionsEnabled.set(enabled);
  try {
    safeStorage?.setItem(KEY, enabled ? "true" : "false");
  } catch {
    // Keep the choice for this session when storage is unavailable.
  }
}
