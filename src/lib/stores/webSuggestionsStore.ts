import { writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";

const KEY = "spark_web_suggestions";

function initial(): boolean {
  try {
    const saved = safeStorage?.getItem(KEY);
    // On unless the user turned it off.
    return saved === null || saved === undefined ? true : saved === "true";
  } catch {
    return true;
  }
}

/** On by default: what is typed in the address bar goes to the search engine while typing (never in a private window). */
export const webSuggestionsEnabled = writable<boolean>(initial());

export function setWebSuggestionsEnabled(enabled: boolean): void {
  webSuggestionsEnabled.set(enabled);
  try {
    safeStorage?.setItem(KEY, enabled ? "true" : "false");
  } catch {
    // Keep the choice for this session when storage is unavailable.
  }
}
