import { get, writable } from "svelte/store";
import { fetchHistory } from "../services/bridge/persistence";
import { buildSuggestions, type Suggestion } from "../services/address-suggestions.js";
import { bookmarks } from "./persistenceStore";
import { addressInput, isAddressEditing, submitNavigation } from "./browserStore";
import { defaultSearchEngine, getSearchEngineLabel } from "./searchEngineStore";

export interface SuggestionState {
  open: boolean;
  items: Suggestion[];
  /** Index of the highlighted row, or -1 when none is (Enter then does what the typed text asks). */
  selected: number;
  /** The most rows shown since the address bar got focus, so the page below does not jump while typing. */
  rows: number;
}

const closed: SuggestionState = { open: false, items: [], selected: -1, rows: 0 };
export const suggestionState = writable<SuggestionState>(closed);

const DEBOUNCE_MS = 70;
const HISTORY_ROWS = 60;
let timer: ReturnType<typeof setTimeout> | undefined;
let generation = 0;

async function compute(query: string, run: number): Promise<void> {
  let history: Awaited<ReturnType<typeof fetchHistory>> = [];
  try {
    history = await fetchHistory(HISTORY_ROWS, query.trim());
  } catch {
    history = [];
  }
  if (run !== generation) return;
  const items = buildSuggestions({
    query,
    history,
    bookmarks: get(bookmarks),
    engineLabel: getSearchEngineLabel(get(defaultSearchEngine)),
  });
  suggestionState.update(state => (state.open ? { ...state, items, selected: -1, rows: Math.max(state.rows, items.length) } : state));
}

/** Shows suggestions for what is typed; with an empty address bar they are the most recent sites. */
export function requestSuggestions(query: string, immediate = false): void {
  const run = ++generation;
  if (timer) clearTimeout(timer);
  suggestionState.update(state => (state.open ? state : { ...state, open: true }));
  if (immediate) void compute(query, run);
  else timer = setTimeout(() => void compute(query, run), DEBOUNCE_MS);
}

export function closeSuggestions(): void {
  generation++;
  if (timer) clearTimeout(timer);
  suggestionState.set(closed);
}

/** Moves the highlight; going past either end returns to "nothing highlighted". */
export function moveSuggestion(delta: 1 | -1): void {
  suggestionState.update(state => {
    if (!state.open || !state.items.length) return state;
    const last = state.items.length - 1;
    let next = state.selected + delta;
    if (next > last) next = -1;
    else if (next < -1) next = last;
    return { ...state, selected: next };
  });
}

export function highlightedSuggestion(): Suggestion | null {
  const state = get(suggestionState);
  return state.open && state.selected >= 0 ? state.items[state.selected] ?? null : null;
}

/** Opens the chosen suggestion: a page goes to its address, a search term or typed text goes through the usual navigation. */
export async function chooseSuggestion(item: Suggestion): Promise<void> {
  closeSuggestions();
  isAddressEditing.set(false);
  addressInput.set(item.target);
  (document.activeElement as HTMLElement | null)?.blur?.();
  await submitNavigation(item.target);
}
