import { createSharedLifecycle } from "../services/lifecycle.js";
import { locale } from "../i18n";
import { derived, get, writable } from "svelte/store";
import {
  onWorkspaceShortcutCancel,
  onWorkspaceShortcutEvent,
  onWorkspaceSwitcherDismissed,
  onWorkspaceSwitcherSelected,
  setWorkspaceSwitcherOverlay,
  type WorkspaceShortcutEvent,
} from "../services/tauri-bridge";
import {
  activateWorkspace,
  activeWorkspaceId,
  workspaces,
} from "./workspaceStore";
import { overlayBlurPx, workspaceSwitcherDisplay } from "./settingsStore";
import { logIgnored } from "../services/ignored-error";

export type WorkspaceSwitcherOverlay = { kind: "picker" };

export const workspaceSwitcherOverlay =
  writable<WorkspaceSwitcherOverlay | null>(null);
export const isWorkspaceSwitcherOverlayOpen = derived(
  workspaceSwitcherOverlay,
  (overlay) => overlay !== null
);

const HOLD_DURATION_MS = 400;

let altIsHeld = false;
let pickerSelectionsInProgress = 0;
let holdTimer: ReturnType<typeof setTimeout> | undefined;

function clearHoldTimer(): void {
  if (holdTimer !== undefined) {
    clearTimeout(holdTimer);
    holdTimer = undefined;
  }
}

function setOverlay(overlay: WorkspaceSwitcherOverlay | null): void {
  if (overlay === null) {
    clearHoldTimer();
    altIsHeld = false;
  }

  const wasOpen = get(workspaceSwitcherOverlay) !== null;
  const willBeOpen = overlay !== null;
  workspaceSwitcherOverlay.set(overlay);

  if (wasOpen !== willBeOpen) {
    void setWorkspaceSwitcherOverlay(
      willBeOpen,
      get(workspaces),
      get(activeWorkspaceId),
      get(overlayBlurPx)
    ).catch(logIgnored);
  }
}

function refreshOpenOverlay(): void {
  if (get(workspaceSwitcherOverlay) === null) return;
  void setWorkspaceSwitcherOverlay(
    true,
    get(workspaces),
    get(activeWorkspaceId),
    get(overlayBlurPx)
  ).catch(logIgnored);
}

function finishPickerSelection(): void {
  if (get(workspaceSwitcherOverlay)?.kind !== "picker") return;
  if (altIsHeld) {
    refreshOpenOverlay();
  } else if (pickerSelectionsInProgress === 0) {
    setOverlay(null);
  }
}

workspaceSwitcherDisplay.subscribe((display) => {
  if (display === "toolbar" && get(workspaceSwitcherOverlay) !== null) {
    setOverlay(null);
  }
});

export function startWorkspaceAltHold(): void {
  if (get(workspaceSwitcherDisplay) !== "popup") {
    cancelWorkspaceAltHoldTimer();
    return;
  }

  if (altIsHeld) return;
  altIsHeld = true;
  clearHoldTimer();
  holdTimer = setTimeout(() => {
    holdTimer = undefined;
    if (altIsHeld) setOverlay({ kind: "picker" });
  }, HOLD_DURATION_MS);
}

/** Cancels the long-press timer when Alt is used with another key. */
export function cancelWorkspaceAltHoldTimer(): void {
  clearHoldTimer();
  altIsHeld = false;
}

export function releaseWorkspaceAlt(): void {
  altIsHeld = false;
  clearHoldTimer();
  if (
    get(workspaceSwitcherOverlay)?.kind === "picker" &&
    pickerSelectionsInProgress === 0
  ) {
    setOverlay(null);
  }
}

export function cancelWorkspaceAltHold(): void {
  altIsHeld = false;
  clearHoldTimer();
  if (get(workspaceSwitcherOverlay)?.kind === "picker") {
    setOverlay(null);
  }
}

export function dismissWorkspacePicker(): void {
  if (get(workspaceSwitcherOverlay)?.kind !== "picker") return;
  clearHoldTimer();
  setOverlay(null);
}

async function selectWorkspaceFromPicker(workspaceId: string): Promise<void> {
  if (get(workspaceSwitcherOverlay)?.kind !== "picker") return;
  clearHoldTimer();
  pickerSelectionsInProgress += 1;
  try {
    await activateWorkspace(workspaceId);
  } finally {
    pickerSelectionsInProgress -= 1;
    finishPickerSelection();
  }
}

async function selectWorkspaceByIndex(index: number): Promise<void> {
  clearHoldTimer();

  if (!Number.isInteger(index) || index < 1 || index > 9) return;
  const target = get(workspaces)[index - 1];
  if (!target) return;

  const pickerWasOpen = get(workspaceSwitcherOverlay)?.kind === "picker";
  if (pickerWasOpen) pickerSelectionsInProgress += 1;
  try {
    await activateWorkspace(target.id);
  } catch {
    // The shortcut is fire-and-forget; avoid an unhandled rejection if switching fails.
  } finally {
    if (pickerWasOpen) {
      pickerSelectionsInProgress -= 1;
      finishPickerSelection();
    }
  }
}

export async function handleWorkspaceShortcutEvent(
  event: WorkspaceShortcutEvent
): Promise<void> {
  if (event.type === "alt-down") {
    startWorkspaceAltHold();
  } else if (event.type === "alt-up") {
    releaseWorkspaceAlt();
  } else if (event.type === "alt-cancel") {
    cancelWorkspaceAltHoldTimer();
  } else if (event.type === "select-index" && event.index !== undefined) {
    await selectWorkspaceByIndex(event.index);
  }
}

function startWorkspaceShortcutBridge(): () => void {
  let disposed = false;
  let unlistenEvent: (() => void) | undefined;
  let unlistenCancel: (() => void) | undefined;
  let unlistenSelection: (() => void) | undefined;
  let unlistenDismiss: (() => void) | undefined;

  void onWorkspaceShortcutEvent((event) => {
    void handleWorkspaceShortcutEvent(event);
  }).then((stopListening) => {
    if (disposed) stopListening();
    else unlistenEvent = stopListening;
  });
  void onWorkspaceShortcutCancel(() => cancelWorkspaceAltHold()).then((stopListening) => {
    if (disposed) stopListening();
    else unlistenCancel = stopListening;
  });
  void onWorkspaceSwitcherSelected((workspaceId) => {
    void chooseWorkspaceFromPicker(workspaceId);
  }).then((stopListening) => {
    if (disposed) stopListening();
    else unlistenSelection = stopListening;
  });
  void onWorkspaceSwitcherDismissed(() => dismissWorkspacePicker()).then((stopListening) => {
    if (disposed) stopListening();
    else unlistenDismiss = stopListening;
  });
  return () => {
    disposed = true;
    unlistenEvent?.();
    unlistenCancel?.();
    unlistenSelection?.();
    unlistenDismiss?.();
    cancelWorkspaceAltHold();
    if (get(workspaceSwitcherOverlay) !== null) setOverlay(null);
  };
}

export async function chooseWorkspaceFromPicker(workspaceId: string): Promise<void> {
  if (!get(workspaces).some((workspace) => workspace.id === workspaceId)) return;
  await selectWorkspaceFromPicker(workspaceId);
}

locale.subscribe(refreshOpenOverlay);

export const initializeWorkspaceShortcutBridge = createSharedLifecycle(scope => scope.own(startWorkspaceShortcutBridge()));
