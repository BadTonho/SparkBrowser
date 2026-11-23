import { get, writable } from "svelte/store";
import { tick } from "svelte";
import { notifyOpening, settleOverlayVisibility } from "../services/bridge/overlays";
import { setToolsPopover } from "../services/bridge/tools-popover";
import { isZenMode } from "./settingsStore";
export type ToolsSection = "sessions" | "tabs" | "notes" | "closed" | "diagnostics" | "feeds";
export const isToolsOpen = writable(false);
export const toolsSection = writable<ToolsSection>("sessions");
export const toolsSession = writable<{ id: number; anchor: HTMLButtonElement } | null>(null);
let revision = 0;
export async function openTools(section: ToolsSection = "sessions", anchor?: HTMLButtonElement) {
  if (get(isToolsOpen) && anchor) return closeTools(true);
  const request = ++revision;
  isZenMode.set(false);
  await tick();
  if (request !== revision) return;
  const visible = (button: HTMLButtonElement | null | undefined) => button?.isConnected && button.getClientRects().length ? button : undefined;
  const reference = visible(anchor) ?? visible(document.querySelector<HTMLButtonElement>('.titlebar-actions [data-action="tools"]')) ??
    visible(document.querySelector<HTMLButtonElement>('[data-testid="open-titlebar-menu"]'));
  if (!reference) return;
  toolsSection.set(section);
  if (get(isToolsOpen)) return;
  notifyOpening("tools");
  await settleOverlayVisibility();
  if (request !== revision || !visible(reference)) return;
  toolsSession.set({ id: request, anchor: reference });
  isToolsOpen.set(true);
}
export async function closeTools(restoreFocus: unknown = false) {
  ++revision;
  const session = get(toolsSession);
  toolsSession.set(null);
  isToolsOpen.set(false);
  if (!session) return;
  await setToolsPopover(false, session.id, undefined, restoreFocus === true);
  if (restoreFocus === true && !get(isToolsOpen) && session.anchor.isConnected && session.anchor.getClientRects().length)
    session.anchor.focus({ preventScroll: true });
}
