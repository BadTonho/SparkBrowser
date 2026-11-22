import { get } from "svelte/store";
import { tick } from "svelte";
import { createPanelProjection, type PanelPresentation } from "../../services/panel-presentation";
import type { Readable } from "svelte/store";
import type { PanelViewAction } from "../../services/panel-presentation";
import { onOverlayOpening } from "../../services/bridge/overlays";

import { locale } from "../../i18n";

export interface PanelSession { id: number; anchor: HTMLElement }
interface HostOptions { id: "tools" | "downloads" | "settings-quick" | "settings" | "suggestions" | "commands" | "resources"; sessionStore: Readable<PanelSession | null>; independent?: boolean; onTrigger?: () => void; close: (restore?: boolean, immediate?: boolean) => Promise<unknown>; set: (visible: boolean, session: number, presentation?: PanelPresentation, focus?: boolean) => Promise<unknown>; listen: (callback: (event: {sessionId: number; sequence: number; action: PanelViewAction}) => void) => Promise<() => void> }

export function observePanelTheme(onChange: () => void): () => void {
  const themeObserver = new MutationObserver(onChange);
  themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["style", "class", "data-theme", "data-palette", "data-window-style"] });
  const colorScheme = window.matchMedia("(prefers-color-scheme: dark)");
  colorScheme.addEventListener("change", onChange);
  return () => { themeObserver.disconnect(); colorScheme.removeEventListener("change", onChange); };
}

export function readGlassFrame(style: CSSStyleDeclaration): Record<string, string> {
  if (document.documentElement.dataset.windowStyle !== "glass") return {};
  return {
    surfaceBorder: style.getPropertyValue("--modal-surface-border").trim(),
    surfaceFilter: style.getPropertyValue("--modal-surface-filter").trim(),
    surfaceShadow: style.getPropertyValue("--modal-surface-shadow").trim(),
  };
}

export function mountPanelHost(root: HTMLElement, native: boolean, options: HostOptions) {
  const { id, sessionStore, close, set, listen } = options;
  const projection = createPanelProjection(root);
  const session = get(sessionStore)!;
  let disposed = false, frame = 0, revision = 0, ack = 0, last = "", dirty = true;
  const observer = new MutationObserver(() => { dirty = true; });
  observer.observe(root, { subtree: true, attributes: true, childList: true, characterData: true });
  let stopActions: (() => void) | undefined;
  let listening = !native;
  const stopOverlay = onOverlayOpening(opening => { if (!options.independent && opening !== id) void close(false, true); });
  const keys = (event: KeyboardEvent) => {
    if (id !== "settings" && event.key === "Escape") { event.preventDefault(); event.stopImmediatePropagation(); void close(true); }
  };
  window.addEventListener("keydown", keys, true);
  const themeKeys = [
    "--text-primary", "--text-secondary", "--text-muted", "--status-on", "--status-on-bg", "--status-on-border",
    "--accent-primary", "--accent-secondary", "--accent-gradient", "--accent-foreground", "--border-subtle",
    "--bg-glass-elevated", "--bg-glass-hover", "--bg-downloads-panel", "--bg-permission-popover",
    "--modal-surface-bg", "--modal-surface-filter", "--modal-surface-border", "--modal-surface-shadow",
    "--settings-backdrop-bg", "--settings-tab-bg", "--settings-row-bg", "--settings-row-border",
    "--settings-control-scheme",
    "--overlay-backdrop-blur",
  ];
  // getComputedStyle forces a style recalculation, so theme values are re-read only
  // after the root element's style, classes or theme attributes change. Theme changes
  // do not set `dirty`: publish still compares the encoded payload, so echoes of a
  // value we just applied do not start another native update.
  let themeChanged = true, theme: Record<string, string> = {};
  const stopTheme = observePanelTheme(() => { themeChanged = true; });
  const readTheme = () => {
    if (themeChanged) {
      const style = getComputedStyle(document.documentElement);
      theme = Object.fromEntries(themeKeys.map(key => [key, style.getPropertyValue(key).trim()]));
      themeChanged = false;
    }
    return theme;
  };
  const publish = () => {
    if (disposed || !listening || get(sessionStore) !== session) return;
    if (!session.anchor.isConnected || !session.anchor.getClientRects().length) { void close(); return; }
    const r = session.anchor.getBoundingClientRect();
    const meta = { anchor: { left: r.left, top: r.top, width: r.width, height: r.height },
      theme: readTheme(), locale: get(locale), ack };
    const encoded = JSON.stringify(meta);
    if (!dirty && encoded === last) return;
    dirty = false;
    last = encoded;
    const payload: Omit<PanelPresentation, "revision"> = { ...meta, tree: projection.snapshot() };
    if (native) void set(true, session.id, { ...payload, revision: ++revision }).catch(() => {
      console.error("Panel presentation failed"); if (!options.independent && get(sessionStore) === session) void close(true);
    });
  };
  const follow = () => { publish(); if (!disposed) frame = requestAnimationFrame(follow); };
  if (native) void listen(async event => {
    if (disposed || get(sessionStore) !== session || event.sessionId !== session.id || event.sequence <= ack) return;
    ack = event.sequence;
    if (event.action.kind === "dismiss") {
      if (event.action.reason === "trigger") options.onTrigger?.();
      await close(["escape", "trigger"].includes(event.action.reason), event.action.reason === "blur"); return;
    }
    projection.dispatch(event.action);
    dirty = true;
    await tick(); publish();
  }).then(stop => { if (disposed) stop(); else { stopActions = stop; listening = true; } });
  frame = requestAnimationFrame(follow);
  return () => { disposed = true; observer.disconnect(); stopTheme(); cancelAnimationFrame(frame); stopActions?.(); stopOverlay(); window.removeEventListener("keydown", keys, true); };
}
