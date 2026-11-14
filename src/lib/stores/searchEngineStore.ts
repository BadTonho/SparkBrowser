import { safeStorage } from "../services/storage.js";
import { writable } from "svelte/store";
import { DEFAULT_SEARCH_ENGINE as DEFAULT_ID, SEARCH_ENGINES, findSearchEngine, isSearchEngineId } from "../services/search-engines.js";

export { SEARCH_ENGINES };

export type SearchEngine = (typeof SEARCH_ENGINES)[number]["id"];

export function getSearchEngineHomeUrl(engine: SearchEngine): string {
  return findSearchEngine(engine).home;
}

export function getSearchEngineLabel(engine: SearchEngine): string {
  return findSearchEngine(engine).label;
}

const SEARCH_ENGINE_STORAGE_KEY = "spark_default_search_engine";
const DEFAULT_SEARCH_ENGINE = DEFAULT_ID as SearchEngine;

function isSearchEngine(value: string | null): value is SearchEngine {
  return isSearchEngineId(value);
}

function getInitialSearchEngine(): SearchEngine {
  if (typeof safeStorage === "undefined") return DEFAULT_SEARCH_ENGINE;

  try {
    const saved = safeStorage.getItem(SEARCH_ENGINE_STORAGE_KEY);
    return isSearchEngine(saved) ? saved : DEFAULT_SEARCH_ENGINE;
  } catch {
    return DEFAULT_SEARCH_ENGINE;
  }
}

export const defaultSearchEngine = writable<SearchEngine>(getInitialSearchEngine());

export function setDefaultSearchEngine(engine: SearchEngine): void {
  const next = isSearchEngine(engine) ? engine : DEFAULT_SEARCH_ENGINE;
  defaultSearchEngine.set(next);

  if (typeof safeStorage === "undefined") return;

  try {
    safeStorage.setItem(SEARCH_ENGINE_STORAGE_KEY, next);
  } catch {
    // Keep the current selection if browser storage is unavailable.
  }
}
