import { derived, get, writable } from "svelte/store";
import {
  onWorkspaceShortcutCancel,
  onWorkspaceShortcutEvent,
  setOverlayVisibility,
  type WorkspaceShortcutEvent,
} from "../services/tauri-bridge";
import { activateWorkspace, workspaces } from "./workspaceStore";

export type WorkspaceSwitcherOverlay =
  | { kind: "picker" }
  | { kind: "confirmation"; workspaceId: string };

export const workspaceSwitcherOverlay =
  writable<WorkspaceSwitcherOverlay | null>(null);
export const isWorkspaceSwitcherOverlayOpen = derived(
  workspaceSwitcherOverlay,
  (overlay) => overlay !== null
);

const HOLD_DURATION_MS = 2_000;
const CONFIRMATION_DURATION_MS = 1_500;

let altIsHeld = false;
let pickerSelectionInProgress = false;
let workspaceSelectionSequence = 0;
let holdTimer: ReturnType<typeof setTimeout> | undefined;
let confirmationTimer: ReturnType<typeof setTimeout> | undefined;

function clearHoldTimer(): void {
  if (holdTimer !== undefined) {
    clearTimeout(holdTimer);
    holdTimer = undefined;
  }
}

function clearConfirmationTimer(): void {
  if (confirmationTimer !== undefined) {
    clearTimeout(confirmationTimer);
    confirmationTimer = undefined;
  }
}

function setOverlay(overlay: WorkspaceSwitcherOverlay | null): void {
  const wasOpen = get(workspaceSwitcherOverlay) !== null;
  const willBeOpen = overlay !== null;
  workspaceSwitcherOverlay.set(overlay);

  if (wasOpen !== willBeOpen) {
    void setOverlayVisibility(willBeOpen, "workspace-switcher");
  }
}

export function startWorkspaceAltHold(): void {
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
    !pickerSelectionInProgress
  ) {
    clearConfirmationTimer();
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
  clearConfirmationTimer();
  pickerSelectionInProgress = true;
  try {
    await activateWorkspace(workspaceId);
  } finally {
    pickerSelectionInProgress = false;
    if (get(workspaceSwitcherOverlay)?.kind === "picker") {
      setOverlay(null);
    }
  }
}

async function selectWorkspaceByIndex(index: number): Promise<void> {
  clearHoldTimer();
  altIsHeld = false;

  if (!Number.isInteger(index) || index < 1 || index > 9) return;
  const target = get(workspaces)[index - 1];
  if (!target) return;

  const selectionSequence = ++workspaceSelectionSequence;
  clearConfirmationTimer();
  setOverlay({ kind: "confirmation", workspaceId: target.id });
  try {
    await activateWorkspace(target.id);
  } catch {
    if (
      selectionSequence === workspaceSelectionSequence &&
      get(workspaceSwitcherOverlay)?.kind === "confirmation"
    ) {
      setOverlay(null);
    }
    return;
  }

  if (selectionSequence !== workspaceSelectionSequence) return;
  clearConfirmationTimer();
  confirmationTimer = setTimeout(() => {
    confirmationTimer = undefined;
    if (get(workspaceSwitcherOverlay)?.kind === "confirmation") {
      setOverlay(null);
    }
  }, CONFIRMATION_DURATION_MS);
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

export function initializeWorkspaceShortcutBridge(): () => void {
  let disposed = false;
  let unlistenEvent: (() => void) | undefined;
  let unlistenCancel: (() => void) | undefined;

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

  return () => {
    disposed = true;
    unlistenEvent?.();
    unlistenCancel?.();
    cancelWorkspaceAltHold();
    clearConfirmationTimer();
    if (get(workspaceSwitcherOverlay) !== null) setOverlay(null);
  };
}

export async function chooseWorkspaceFromPicker(workspaceId: string): Promise<void> {
  if (!get(workspaces).some((workspace) => workspace.id === workspaceId)) return;
  await selectWorkspaceFromPicker(workspaceId);
}
