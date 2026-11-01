import { writable } from "svelte/store";

export const SEARCH_ENGINES = [
  { id: "google", label: "Google" },
  { id: "duckduckgo", label: "DuckDuckGo" },
] as const;

export type SearchEngine = (typeof SEARCH_ENGINES)[number]["id"];

const SEARCH_ENGINE_HOME_URLS: Record<SearchEngine, string> = {
  google: "https://www.google.com/",
  duckduckgo: "https://duckduckgo.com/",
};

export function getSearchEngineHomeUrl(engine: SearchEngine): string {
  return SEARCH_ENGINE_HOME_URLS[engine];
}

export function getSearchEngineLabel(engine: SearchEngine): string {
  return SEARCH_ENGINES.find((item) => item.id === engine)?.label ?? "Google";
}

const SEARCH_ENGINE_STORAGE_KEY = "veloce_default_search_engine";
const DEFAULT_SEARCH_ENGINE: SearchEngine = "google";

function isSearchEngine(value: string | null): value is SearchEngine {
  return SEARCH_ENGINES.some((engine) => engine.id === value);
}

function getInitialSearchEngine(): SearchEngine {
  if (typeof localStorage === "undefined") return DEFAULT_SEARCH_ENGINE;

  try {
    const saved = localStorage.getItem(SEARCH_ENGINE_STORAGE_KEY);
    return isSearchEngine(saved) ? saved : DEFAULT_SEARCH_ENGINE;
  } catch {
    return DEFAULT_SEARCH_ENGINE;
  }
}

export const defaultSearchEngine = writable<SearchEngine>(getInitialSearchEngine());

export function setDefaultSearchEngine(engine: SearchEngine): void {
  const next = isSearchEngine(engine) ? engine : DEFAULT_SEARCH_ENGINE;
  defaultSearchEngine.set(next);

  if (typeof localStorage === "undefined") return;

  try {
    localStorage.setItem(SEARCH_ENGINE_STORAGE_KEY, next);
  } catch {
    // Keep the current selection if browser storage is unavailable.
  }
}
