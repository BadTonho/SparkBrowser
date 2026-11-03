import { get } from "svelte/store";
import { locale, tr } from "../../i18n";
import { invoke } from "./runtime";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { WorkspaceItem } from "../../types/browser";

import { isTauriRuntime } from "./runtime";

export type OverlayId =
  | "update-install"
  | "shutdown"
  | "site-permissions"
  | "permission-prompt"
  | "tools"
  | "command-bar"
  | "history"
  | "downloads"
  | "settings"
  | "titlebar-menu"
  | "bookmark-editor";

const activeOverlayIds = new Set<OverlayId>();
let overlaySyncQueue: Promise<void> = Promise.resolve();

const openingListeners = new Set<(id?: OverlayId) => void>();
export function onOverlayOpening(callback: (id?: OverlayId) => void): () => void {
  openingListeners.add(callback);
  return () => { openingListeners.delete(callback); };
}
function notifyOpening(id?: OverlayId) { for (const callback of openingListeners) callback(id); }

/** Keeps the native content WebView hidden while any browser overlay is open. */
export async function setOverlayVisibility(
  visible: boolean,
  overlayId: OverlayId
): Promise<void> {
  if(visible) {
    activeOverlayIds.add(overlayId);
    // Register the destination before listeners dismiss the previous overlay.
    notifyOpening(overlayId);
  } else {
    activeOverlayIds.delete(overlayId);
  }

  if(!isTauriRuntime()) return;

  const shouldHideContent = activeOverlayIds.size > 0;
  overlaySyncQueue = overlaySyncQueue
    .catch(() => undefined)
    .then(() => invoke("set_overlay_visibility", { visible: shouldHideContent }));
  await overlaySyncQueue;
}

let workspaceSwitcherSyncQueue: Promise<void> = Promise.resolve();

/** Shows the workspace picker in its transparent native overlay WebView. */
export async function setWorkspaceSwitcherOverlay(
  visible: boolean,
  workspaces: WorkspaceItem[],
  activeWorkspaceId: string,
  backdropBlurPx: number
): Promise<void> {
  if (visible) notifyOpening();
  if(!isTauriRuntime()) return;

  workspaceSwitcherSyncQueue = workspaceSwitcherSyncQueue
    .catch(() => undefined)
    .then(() =>
      invoke<void>("set_workspace_switcher_overlay", {
        visible,
        workspaces,
        activeWorkspaceId,
        backdropBlurPx,
        language: { locale: get(locale), label: tr("workspaces.choose_a_workspace_995c") },
      })
    );
  await workspaceSwitcherSyncQueue;
}

export interface BookmarkContextMenuAction {
  id: string;
  label: string;
  danger?: boolean;
}

let bookmarkContextMenuSyncQueue: Promise<void> = Promise.resolve();

export async function setBookmarkContextMenuOverlay(
  visible: boolean,
  x = 8,
  y = 8,
  actions: BookmarkContextMenuAction[] = [],
  backdropBlurPx = 16
): Promise<void> {
  if (visible) notifyOpening();
  if(!isTauriRuntime()) return;
  bookmarkContextMenuSyncQueue = bookmarkContextMenuSyncQueue
    .catch(() => undefined)
    .then(() => invoke<void>("set_bookmark_context_menu_overlay", {
      visible,
      x,
      y,
      actions,
      presentation: { backdropBlurPx, language: { locale: get(locale), label: tr("common.bookmark_actions") } },
    }));
  await bookmarkContextMenuSyncQueue;
}

export async function onBookmarkContextMenuAction(
  callback: (action: string) => void
): Promise<UnlistenFn> {
  if(!isTauriRuntime()) return () => { };
  return await listen<string>("bookmark-context-menu-action", (event) => {
    callback(event.payload);
  });
}
