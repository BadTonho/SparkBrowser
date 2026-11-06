import { get } from "svelte/store";
import { tick } from "svelte";
import { createPanelProjection, type PanelPresentation } from "../../services/panel-presentation";
import type { Readable } from "svelte/store";
import type { PanelViewAction } from "../../services/panel-presentation";
import { onOverlayOpening } from "../../services/bridge/overlays";

import { locale } from "../../i18n";

export interface PanelSession { id: number; anchor: HTMLButtonElement }
interface HostOptions { id: "tools" | "downloads" | "settings-quick" | "settings"; sessionStore: Readable<PanelSession | null>; onTrigger?: () => void; close: (restore?: boolean) => Promise<unknown>; set: (visible: boolean, session: number, presentation?: PanelPresentation, focus?: boolean) => Promise<unknown>; listen: (callback: (event: {sessionId: number; sequence: number; action: PanelViewAction}) => void) => Promise<() => void> }
export function mountPanelHost(root: HTMLElement, native: boolean, options: HostOptions) {
  const { id, sessionStore, close, set, listen } = options;
  const projection = createPanelProjection(root);
  const session = get(sessionStore)!;
  let disposed = false, frame = 0, revision = 0, ack = 0, last = "", dirty = true;
  const observer = new MutationObserver(() => { dirty = true; });
  observer.observe(root, { subtree: true, attributes: true, childList: true, characterData: true });
  let stopActions: (() => void) | undefined;
  let listening = !native;
  const stopOverlay = onOverlayOpening(opening => { if (opening !== id) void close(); });
  const keys = (event: KeyboardEvent) => {
    if (id !== "settings" && event.key === "Escape") { event.preventDefault(); event.stopImmediatePropagation(); void close(true); }
  };
  window.addEventListener("keydown", keys, true);
  const themeKeys = ["--text-primary", "--text-secondary", "--text-muted", "--accent-primary", "--accent-foreground", "--border-subtle", "--bg-glass-elevated", "--bg-glass-hover", "--bg-downloads-panel", "--bg-permission-popover", "--modal-surface-bg", "--modal-surface-filter", "--modal-surface-border", "--modal-surface-shadow", "--overlay-backdrop-blur"];
  const publish = () => {
    if (disposed || !listening || get(sessionStore) !== session) return;
    if (!session.anchor.isConnected || !session.anchor.getClientRects().length) { void close(); return; }
    const r = session.anchor.getBoundingClientRect(), style = getComputedStyle(document.documentElement);
    const meta = { anchor: { left: r.left, top: r.top, width: r.width, height: r.height },
      theme: Object.fromEntries(themeKeys.map(key => [key, style.getPropertyValue(key).trim()])), locale: get(locale), ack };
    const encoded = JSON.stringify(meta);
    if (!dirty && encoded === last) return;
    dirty = false;
    last = encoded;
    const payload: Omit<PanelPresentation, "revision"> = { ...meta, tree: projection.snapshot() };
    if (native) void set(true, session.id, { ...payload, revision: ++revision }).catch(() => {
      console.error("Panel presentation failed"); if (get(sessionStore) === session) void close(true);
    });
  };
  const follow = () => { publish(); if (!disposed) frame = requestAnimationFrame(follow); };
  if (native) void listen(async event => {
    if (disposed || get(sessionStore) !== session || event.sessionId !== session.id || event.sequence <= ack) return;
    ack = event.sequence;
    if (event.action.kind === "dismiss") {
      if (event.action.reason === "trigger") options.onTrigger?.();
      await close(["escape", "trigger"].includes(event.action.reason)); return;
    }
    projection.dispatch(event.action);
    dirty = true;
    await tick(); publish();
  }).then(stop => { if (disposed) stop(); else { stopActions = stop; listening = true; } });
  frame = requestAnimationFrame(follow);
  return () => { disposed = true; observer.disconnect(); cancelAnimationFrame(frame); stopActions?.(); stopOverlay(); window.removeEventListener("keydown", keys, true); };
}
