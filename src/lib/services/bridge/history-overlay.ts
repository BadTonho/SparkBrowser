import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";

export interface HistoryOverlayPresentation {
  tabId: string;
  url: string;
  locale: "pt-BR" | "en";
  labels: { title: string; clear: string; close: string; search: string; empty: string; loadMore: string };
  theme: { surface: string; border: string; text: string; muted: string; accent: string; hover: string; field: string };
  query: string;
  entries: Array<{ id: number; title: string; url: string; time: string }>;
  busy: boolean;
  error: string;
  hasMore: boolean;
}

export interface HistoryOverlayAction {
  sessionId: number;
  tabId: string;
  url: string;
  action: "close" | "search" | "clear" | "load-more" | "open";
  query: string;
  entryId: number | null;
  entryUrl: string | null;
}

let queue: Promise<void> = Promise.resolve();
export function setHistoryOverlay(visible: boolean, sessionId: number, presentation?: HistoryOverlayPresentation): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_history_overlay", { visible, sessionId, presentation: presentation ?? null }));
  return queue;
}

export const onHistoryOverlayAction = (callback: (action: HistoryOverlayAction) => void): Promise<UnlistenFn> =>
  isTauriRuntime() ? listen<HistoryOverlayAction>("history-overlay-action", event => callback(event.payload)) : Promise.resolve(() => {});
