import { get } from "svelte/store";
import { tick } from "svelte";
import { createToolsProjection, type ToolsPresentation } from "../../services/tools-presentation";
import { onToolsPopoverAction, setToolsPopover } from "../../services/bridge/tools-popover";
import { onOverlayOpening } from "../../services/bridge/overlays";
import { toolsSession, closeTools } from "../../stores/toolsStore";
import { locale } from "../../i18n";

export function mountToolsHost(root: HTMLElement, native: boolean) {
  const projection = createToolsProjection(root);
  const session = get(toolsSession)!;
  let disposed = false, frame = 0, revision = 0, ack = 0, last = "", dirty = true;
  const observer = new MutationObserver(() => { dirty = true; });
  observer.observe(root, { subtree: true, attributes: true, childList: true, characterData: true });
  let stopActions: (() => void) | undefined;
  let listening = !native;
  const stopOverlay = onOverlayOpening(id => { if (id !== "tools") void closeTools(); });
  const keys = (event: KeyboardEvent) => {
    if (event.key === "Escape") { event.preventDefault(); event.stopImmediatePropagation(); void closeTools(true); }
  };
  window.addEventListener("keydown", keys, true);
  const themeKeys = ["--text-primary", "--text-secondary", "--text-muted", "--accent-primary", "--accent-foreground", "--border-subtle"];
  const publish = () => {
    if (disposed || !listening || get(toolsSession) !== session) return;
    if (!session.anchor.isConnected || !session.anchor.getClientRects().length) { void closeTools(); return; }
    const r = session.anchor.getBoundingClientRect(), style = getComputedStyle(document.documentElement);
    const meta = { anchor: { left: r.left, top: r.top, width: r.width, height: r.height },
      theme: Object.fromEntries(themeKeys.map(key => [key, style.getPropertyValue(key).trim()])), locale: get(locale), ack };
    const encoded = JSON.stringify(meta);
    if (!dirty && encoded === last) return;
    dirty = false;
    last = encoded;
    const payload: Omit<ToolsPresentation, "revision"> = { ...meta, tree: projection.snapshot() };
    if (native) void setToolsPopover(true, session.id, { ...payload, revision: ++revision }).catch(error => {
      console.error("Tools presentation failed", error); if (get(toolsSession) === session) void closeTools(true);
    });
  };
  const follow = () => { publish(); if (!disposed) frame = requestAnimationFrame(follow); };
  if (native) void onToolsPopoverAction(async event => {
    if (disposed || get(toolsSession) !== session || event.sessionId !== session.id || event.sequence <= ack) return;
    ack = event.sequence;
    if (event.action.kind === "dismiss") { await closeTools(["escape", "trigger"].includes(event.action.reason)); return; }
    projection.dispatch(event.action);
    dirty = true;
    await tick(); publish();
  }).then(stop => { if (disposed) stop(); else { stopActions = stop; listening = true; } });
  frame = requestAnimationFrame(follow);
  return () => { disposed = true; observer.disconnect(); cancelAnimationFrame(frame); stopActions?.(); stopOverlay(); window.removeEventListener("keydown", keys, true); };
}
