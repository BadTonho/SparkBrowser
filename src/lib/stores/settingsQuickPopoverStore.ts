import { get, writable } from "svelte/store";
import { tick } from "svelte";
import { notifyOpening } from "../services/bridge/overlays";
import { setSettingsQuickPopover } from "../services/bridge/settings-quick-popover";

export interface SettingsQuickSession { id: number; anchor: HTMLButtonElement }

export const isSettingsQuickOpen = writable(false);
export const settingsQuickSession = writable<SettingsQuickSession | null>(null);
let revision = 0;

export async function openSettingsQuickMenu(anchor?: HTMLButtonElement): Promise<void> {
  if (get(isSettingsQuickOpen)) {
    await closeSettingsQuickMenu(true);
    return;
  }
  const request = ++revision;
  await tick();
  if (request !== revision) return;
  const visible = (button: HTMLButtonElement | null | undefined) => button?.isConnected && button.getClientRects().length ? button : undefined;
  const reference = visible(anchor) ?? visible(document.querySelector<HTMLButtonElement>('.titlebar-actions [data-action="settings"]'));
  if (!reference) return;
  notifyOpening("settings-quick");
  if (request !== revision || get(isSettingsQuickOpen)) return;
  settingsQuickSession.set({ id: request, anchor: reference });
  isSettingsQuickOpen.set(true);
}

export async function closeSettingsQuickMenu(restoreFocus = false): Promise<void> {
  ++revision;
  const session = get(settingsQuickSession);
  settingsQuickSession.set(null);
  isSettingsQuickOpen.set(false);
  if (!session) return;
  await setSettingsQuickPopover(false, session.id, undefined, restoreFocus);
  if (restoreFocus && !get(isSettingsQuickOpen) && session.anchor.isConnected && session.anchor.getClientRects().length) {
    session.anchor.focus({ preventScroll: true });
  }
}
