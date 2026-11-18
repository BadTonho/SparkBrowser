import { get, writable } from "svelte/store";
import { spring } from "svelte/motion";
import { setOverlayVisibility } from "../services/tauri-bridge";
import type { PanelSession } from "../components/panels/panel-host";
import { setCommandsPopover } from "../services/bridge/commands-popover";

export const isCommandBarOpen = writable<boolean>(false);
export const commandQuery = writable<string>("");

/**
 * Inside the app the bar is drawn by a native layer above the page. This session ties the real bar, which
 * stays in the shell and holds all the logic, to that layer.
 */
export const commandBarSession = writable<PanelSession | null>(null);
let sessionCounter = 0;
export function beginCommandBarSession(anchor: HTMLElement): void {
  if (get(commandBarSession)) return;
  commandBarSession.set({ id: ++sessionCounter, anchor });
}
export function endCommandBarSession(): void {
  const session = get(commandBarSession);
  if (!session) return;
  commandBarSession.set(null);
  // Focus goes back to the browser window, so the keyboard is never left in the layer that just closed.
  void setCommandsPopover(false, session.id, undefined, true).catch(() => {});
}

/**
 * Spring-physics scale for the floating Command Bar modal.
 */
export const commandBarScale = spring<number>(0.92, {
  stiffness: 0.22,
  damping: 0.68,
});

export async function openCommandBar(initialQuery = ""): Promise<void> {
  commandQuery.set(initialQuery);
  isCommandBarOpen.set(true);
  commandBarScale.set(1);
  await setOverlayVisibility(true, "command-bar");
}

export async function closeCommandBar(): Promise<void> {
  commandBarScale.set(0.92);
  isCommandBarOpen.set(false);
  commandQuery.set("");
  await setOverlayVisibility(false, "command-bar");
}

export async function toggleCommandBar(): Promise<void> {
  let currentlyOpen = false;
  isCommandBarOpen.subscribe((val) => {
    currentlyOpen = val;
  })();

  if (currentlyOpen) {
    await closeCommandBar();
  } else {
    await openCommandBar();
  }
}

