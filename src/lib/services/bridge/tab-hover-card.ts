import { invoke, isTauriRuntime } from "./runtime";

export type HoverCardPlacement = "below" | "right";
export interface HoverCard {
  tabId: string; title: string; memory: string; noPreview: string; locale: string;
  placement: HoverCardPlacement;
  anchor: { x: number; y: number; width: number; height: number };
}

let revision = Date.now(), queue: Promise<void> = Promise.resolve();

/** Shows the hover card of a tab, or hides it with `null`. Only the latest request is sent. */
export function setTabHoverCard(card: HoverCard | null): Promise<void> {
  const current = ++revision;
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => undefined).then(() => current === revision
    ? invoke<void>("set_tab_hover_card", { revision: current, card }) : undefined);
  return queue;
}

/** Takes new pictures of the pages on screen, for the hover cards of tabs left afterwards. */
export const refreshTabPreviews = (): Promise<void> =>
  isTauriRuntime() ? invoke<void>("refresh_tab_previews") : Promise.resolve();
