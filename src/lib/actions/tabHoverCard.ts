import { get } from "svelte/store";
import { tabDrag } from "../stores/tabDragStore";
import { locale } from "../i18n";
import { logIgnored } from "../services/ignored-error";
import { refreshTabPreviews, setTabHoverCard, type HoverCardPlacement } from "../services/bridge/tab-hover-card";

export interface TabHoverCardOptions {
  tabId: string;
  title: string;
  memory: string;
  noPreview: string;
  placement: HoverCardPlacement;
}

/** Wait before the first card, so moving the pointer across the tabs does not flash cards. */
const SHOW_DELAY = 450;
/** Right after a card closes, the next tab shows its card at once (moving along the tabs). */
const QUICK_SWITCH = 350;
/** Pages on screen are pictured at most this often while the pointer is over the tabs. */
const REFRESH_INTERVAL = 3000;
const MAX_TITLE = 60;

let owner: symbol | null = null;
let lastHidden = 0;
let lastRefresh = 0;

/** The tab title, shortened to one line. */
export function shortTitle(title: string, max = MAX_TITLE): string {
  const clean = title.replace(/\s+/g, " ").trim();
  return clean.length > max ? `${clean.slice(0, max - 1).trimEnd()}…` : clean;
}

function report(error: unknown) {
  logIgnored(new Error(`Tab hover card failed: ${error instanceof Error ? error.message : String(error)}`));
}

/** Pictures the visible pages, so the tab the user is about to leave keeps a recent picture. */
export function refreshVisiblePreviews(force = false) {
  const now = Date.now();
  if (!force && now - lastRefresh < REFRESH_INTERVAL) return;
  lastRefresh = now;
  refreshTabPreviews().catch(report);
}

/**
 * Shows the native hover card of a tab (title, page picture and memory) while the pointer rests on
 * it. Pressing, scrolling, dragging or leaving the tab hides it.
 */
export function tabHoverCard(node: HTMLElement, initial: TabHoverCardOptions) {
  const id = Symbol("tab-hover-card");
  let options = initial, timer: ReturnType<typeof setTimeout> | undefined;

  const send = () => {
    if (owner !== id) return;
    const rect = node.getBoundingClientRect();
    setTabHoverCard({
      tabId: options.tabId,
      title: shortTitle(options.title),
      memory: options.memory,
      noPreview: options.noPreview,
      locale: get(locale),
      placement: options.placement,
      anchor: { x: rect.left, y: rect.top, width: rect.width, height: rect.height },
    }).catch(report);
  };
  const show = () => {
    timer = undefined;
    if (get(tabDrag) || !node.isConnected) return;
    owner = id;
    send();
  };
  const hide = () => {
    if (timer) clearTimeout(timer);
    timer = undefined;
    if (owner !== id) return;
    owner = null;
    lastHidden = Date.now();
    setTabHoverCard(null).catch(report);
  };
  const enter = (event: PointerEvent) => {
    if (event.pointerType === "touch" || event.buttons !== 0 || get(tabDrag)) return;
    refreshVisiblePreviews();
    if (timer) clearTimeout(timer);
    timer = setTimeout(show, Date.now() - lastHidden < QUICK_SWITCH ? 0 : SHOW_DELAY);
  };

  node.addEventListener("pointerenter", enter);
  node.addEventListener("pointerleave", hide);
  node.addEventListener("pointerdown", hide);
  node.addEventListener("wheel", hide, { passive: true });
  node.addEventListener("contextmenu", hide);
  window.addEventListener("blur", hide);

  return {
    update(next: TabHoverCardOptions) {
      const changed = next.tabId !== options.tabId || next.title !== options.title
        || next.memory !== options.memory || next.placement !== options.placement || next.noPreview !== options.noPreview;
      options = next;
      if (changed && owner === id) send();
    },
    destroy() {
      hide();
      node.removeEventListener("pointerenter", enter);
      node.removeEventListener("pointerleave", hide);
      node.removeEventListener("pointerdown", hide);
      node.removeEventListener("wheel", hide);
      node.removeEventListener("contextmenu", hide);
      window.removeEventListener("blur", hide);
    },
  };
}
