import { get, writable } from "svelte/store";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { setOverlayVisibility } from "../services/bridge/overlays";

interface MenuSession { anchor: HTMLButtonElement; workspaceId: string }
export const titlebarMenu = writable<MenuSession | null>(null);
let pending: MenuSession | null = null;
let revision = 0;

export async function openTitlebarMenu(anchor: HTMLButtonElement): Promise<void> {
  if (get(titlebarMenu) || pending) return closeTitlebarMenu(true);
  const request = ++revision;
  pending = { anchor, workspaceId: get(activeWorkspaceId) };
  try {
    await setOverlayVisibility(true, "titlebar-menu");
    if (request !== revision) return;
    if (!anchor.isConnected || !anchor.getClientRects().length) return closeTitlebarMenu();
    titlebarMenu.set(pending);
    pending = null;
  } catch (error) {
    if (request === revision) await closeTitlebarMenu();
    throw error;
  }
}

export async function closeTitlebarMenu(restoreFocus = false): Promise<void> {
  const request = ++revision;
  const session = get(titlebarMenu) ?? pending;
  if (!session) return;
  pending = null;
  titlebarMenu.set(null);
  await setOverlayVisibility(false, "titlebar-menu");
  if (restoreFocus && request === revision && session.anchor.isConnected && session.anchor.getClientRects().length)
    session.anchor.focus({ preventScroll: true });
}
