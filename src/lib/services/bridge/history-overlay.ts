import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";

export interface HistoryOverlayPresentation {
  tabId: string;
  url: string;
  locale: "pt-BR" | "en";
  labels: {
    title: string; clear: string; close: string; search: string; empty: string; loadMore: string;
    /** "{count} selected". */
    selected: string; open: string; delete: string; cancelSelection: string; moreFromSite: string; deleteEntry: string; menu: string; select: string;
  };
  theme: {
    surface: string; border: string; text: string; muted: string; accent: string; hover: string; field: string;
    surfaceBorder?: string; surfaceFilter?: string; surfaceShadow?: string;
  };
  query: string;
  /** `day` is the heading of the visit's day; `gap` marks a pause of more than half an hour since the newer visit. */
  entries: Array<{ id: number; title: string; url: string; time: string; host: string; day: string; gap: boolean; site: string | null }>;
  /** Site icons (`data:image/` addresses) by site key. */
  icons: Record<string, string>;
  busy: boolean;
  error: string;
  hasMore: boolean;
}

export interface HistoryOverlayAction {
  sessionId: number;
  tabId: string;
  url: string;
  action: "close" | "search" | "clear" | "load-more" | "open" | "open-many" | "delete" | "more-from-site";
  query: string;
  entryId: number | null;
  entryUrl: string | null;
  entryIds: number[];
  entryUrls: string[];
  entryHost: string | null;
}

let queue: Promise<void> = Promise.resolve();
export function setHistoryOverlay(visible: boolean, sessionId: number, presentation?: HistoryOverlayPresentation): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_history_overlay", { visible, sessionId, presentation: presentation ?? null }));
  return queue;
}

export const onHistoryOverlayAction = (callback: (action: HistoryOverlayAction) => void): Promise<UnlistenFn> =>
  isTauriRuntime() ? listen<HistoryOverlayAction>("history-overlay-action", event => callback(event.payload)) : Promise.resolve(() => {});

/** Sends the rows to the history tab (Settings → History display → Tab). */
export const setHistoryTab = (tabId: string, sessionId: number, presentation: HistoryOverlayPresentation): Promise<void> =>
  isTauriRuntime() ? invoke<void>("set_history_tab", { tabId, sessionId, presentation }) : Promise.resolve();
export const onHistoryTabAction = (callback: (action: HistoryOverlayAction) => void): Promise<UnlistenFn> =>
  isTauriRuntime() ? listen<HistoryOverlayAction>("history-tab-action", event => callback(event.payload)) : Promise.resolve(() => {});
/** The history page finished loading in a tab (its id): it is ready for its rows. */
export const onHistoryTabLoaded = (callback: (tabId: string) => void): Promise<UnlistenFn> =>
  isTauriRuntime() ? listen<string>("history-tab-loaded", event => callback(event.payload)) : Promise.resolve(() => {});
