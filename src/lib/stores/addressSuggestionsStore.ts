import { get, writable } from "svelte/store";
import { fetchHistory } from "../services/bridge/persistence";
import { buildSuggestions, looksLikeAddress, supportsWebSuggestions, type Suggestion } from "../services/address-suggestions.js";
import { fetchSearchSuggestions } from "../services/bridge/search-suggestions";
import { bookmarks } from "./persistenceStore";
import { setSuggestionsPopover } from "../services/bridge/suggestions-popover";
import { logIgnored } from "../services/ignored-error";
import type { PanelSession } from "../components/panels/panel-host";
import { addressInput, isAddressEditing, submitNavigation } from "./browserStore";
import { defaultSearchEngine, getSearchEngineLabel } from "./searchEngineStore";
import { webSuggestionsEnabled } from "./webSuggestionsStore";
import { isPrivateWindow } from "./privateWindowStore";

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

/**
 * In the app the list is a native popover over the page (below the address bar). A session lives while the
 * popover is shown; it ends when there is nothing to list or the list closes.
 */
export const suggestionsSession = writable<PanelSession | null>(null);
let sessionCounter = 0;

export function beginSuggestionsSession(anchor: HTMLElement): void {
  if (get(suggestionsSession)) return;
  suggestionsSession.set({ id: ++sessionCounter, anchor });
}

export function endSuggestionsSession(): void {
  const session = get(suggestionsSession);
  if (!session) return;
  suggestionsSession.set(null);
  void setSuggestionsPopover(false, session.id).catch(logIgnored);
}

const DEBOUNCE_MS = 70;
const HISTORY_ROWS = 60;
let timer: ReturnType<typeof setTimeout> | undefined;
let generation = 0;

const WEB_DELAY_MS = 150;
const delay = (ms: number) => new Promise<void>(resolve => setTimeout(resolve, ms));
const termsFor = new Map<string, string[]>();

async function compute(query: string, run: number): Promise<void> {
  let history: Awaited<ReturnType<typeof fetchHistory>> = [];
  try {
    history = await fetchHistory(HISTORY_ROWS, query.trim());
  } catch {
    history = [];
  }
  if (run !== generation) return;
  const engine = get(defaultSearchEngine);
  const text = query.trim();
  const show = (webTerms: string[], keepHighlight: boolean) => {
    const items = buildSuggestions({
      query,
      history,
      bookmarks: get(bookmarks),
      webTerms,
      engineLabel: getSearchEngineLabel(engine),
    });
    suggestionState.update(state => {
      if (!state.open) return state;
      const kept = keepHighlight && state.selected >= 0 ? state.items[state.selected]?.id : undefined;
      const selected = kept ? items.findIndex(item => item.id === kept) : -1;
      return { ...state, items, selected, rows: Math.max(state.rows, items.length) };
    });
  };
  const key = `${engine}
${text.toLowerCase()}`;
  show(termsFor.get(key) ?? [], false);

  // Terms from the search engine need the option on, a normal window, words (not an address) and an engine that has them.
  const wantsWeb = get(webSuggestionsEnabled) && !get(isPrivateWindow) && text.length > 0 && !looksLikeAddress(text) && supportsWebSuggestions(engine);
  if (!wantsWeb || termsFor.has(key)) return;
  await delay(WEB_DELAY_MS);
  if (run !== generation) return;
  const terms = await fetchSearchSuggestions(engine, text);
  if (run !== generation) return;
  if (termsFor.size >= 50) termsFor.clear();
  termsFor.set(key, terms);
  show(terms, true);
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
  endSuggestionsSession();
  generation++;
  termsFor.clear();
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
