import { get, writable } from "svelte/store";
import { tick } from "svelte";
import { notifyOpening } from "../services/bridge/overlays";
import { setResourcesPopover } from "../services/bridge/resources-popover";

export interface ResourcesSession { id: number; anchor: HTMLButtonElement }

export const isResourcesOpen = writable(false);
export const resourcesSession = writable<ResourcesSession | null>(null);
let revision = 0;

export async function openResourcesPanel(anchor?: HTMLButtonElement): Promise<void> {
  if (get(isResourcesOpen)) {
    await closeResourcesPanel(true);
    return;
  }
  const request = ++revision;
  await tick();
  if (request !== revision) return;
  const visible = (button: HTMLButtonElement | null | undefined) => button?.isConnected && button.getClientRects().length ? button : undefined;
  const reference = visible(anchor) ?? visible(document.querySelector<HTMLButtonElement>('.titlebar-actions [data-action="resources"]'));
  if (!reference) return;
  notifyOpening("resources");
  if (request !== revision || get(isResourcesOpen)) return;
  resourcesSession.set({ id: request, anchor: reference });
  isResourcesOpen.set(true);
}

export async function closeResourcesPanel(restoreFocus = false): Promise<void> {
  ++revision;
  const session = get(resourcesSession);
  resourcesSession.set(null);
  isResourcesOpen.set(false);
  if (!session) return;
  await setResourcesPopover(false, session.id, undefined, restoreFocus);
  if (restoreFocus && !get(isResourcesOpen) && session.anchor.isConnected && session.anchor.getClientRects().length) {
    session.anchor.focus({ preventScroll: true });
  }
}
