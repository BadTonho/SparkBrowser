import { tr, locale } from "../i18n";
import { describeError } from "../services/user-errors.js";
import { writable, derived, get } from "svelte/store";
import type { BookmarkData, BookmarkFolder, BookmarkItem, HistoryEntry } from "../types/browser";
import {
  fetchBookmarkData,
  fetchHistory,
  clearAllHistory,
  saveBookmarkItem,
  removeBookmarkItem,
  createBookmarkFolder as createBookmarkFolderCommand,
  renameBookmarkFolder as renameBookmarkFolderCommand,
  deleteBookmarkFolder as deleteBookmarkFolderCommand,
  setBookmarkContextMenuOverlay,
  setOverlayVisibility,
  isTauriRuntime,
  currentWindowId,
  getBrowserWindows,
  windowNumber,
} from "../services/tauri-bridge";
import { overlayBlurPx } from "./settingsStore";

export type BookmarkEditorState =
  | {
      kind: "save";
      bookmark: BookmarkItem | null;
      title: string;
      url: string;
      icon: string;
    }
  | { kind: "folders" }
  | { kind: "create-folder" }
  | { kind: "rename-folder"; folderId: number; name: string }
  | null;

export type BookmarkContextMenuState =
  | { kind: "bookmark"; x: number; y: number; bookmark: BookmarkItem }
  | { kind: "folder"; x: number; y: number; folder: BookmarkFolder }
  | { kind: "empty"; x: number; y: number }
  | { kind: "tab"; x: number; y: number; tabId: string; windows: string[] }
  | null;

export const bookmarks = writable<BookmarkItem[]>([]);
export const bookmarkFolders = writable<BookmarkFolder[]>([]);
export const bookmarkEditorState = writable<BookmarkEditorState>(null);
export const bookmarkContextMenuState = writable<BookmarkContextMenuState>(null);
export const historyEntries = writable<HistoryEntry[]>([]);
export const isHistoryDrawerOpen = writable<boolean>(false);
export const historyHasMore = writable(false);
export const historyBusy = writable(false);
export const historyError = writable("");
let historyQuery = "";
let historyRevision = 0;

export const bookmarksByFolder = derived(bookmarks, items => {
  const groups = new Map<number | null, BookmarkItem[]>();
  for (const item of items) {
    const id = item.folder_id ?? null;
    const group = groups.get(id) ?? [];
    group.push(item);
    groups.set(id, group);
  }
  return groups;
});

function applyBookmarkData(data: BookmarkData): void {
  bookmarks.set(data.bookmarks);
  bookmarkFolders.set(data.folders);
}

export async function loadBookmarks(): Promise<void> {
  applyBookmarkData(await fetchBookmarkData());
}

export async function openBookmarkEditor(title: string, url: string, icon = "star"): Promise<void> {
  const existing = get(bookmarks).find((item) => item.url === url) ?? null;
  bookmarkEditorState.set({
    kind: "save",
    bookmark: existing,
    title: existing?.title ?? title,
    url,
    icon: existing?.icon ?? icon,
  });
  await setOverlayVisibility(true, "bookmark-editor");
}

export async function openBookmarkFolderManager(): Promise<void> {
  bookmarkEditorState.set({ kind: "folders" });
  await setOverlayVisibility(true, "bookmark-editor");
}

export async function openCreateBookmarkFolder(): Promise<void> {
  bookmarkEditorState.set({ kind: "create-folder" });
  await setOverlayVisibility(true, "bookmark-editor");
}

export async function openBookmarkFolderEditor(folder: BookmarkFolder): Promise<void> {
  bookmarkEditorState.set({ kind: "rename-folder", folderId: folder.id, name: folder.name });
  await setOverlayVisibility(true, "bookmark-editor");
}

export function showBookmarkContextMenu(
  event: MouseEvent,
  target:
    | { kind: "bookmark"; bookmark: BookmarkItem }
    | { kind: "folder"; folder: BookmarkFolder }
    | { kind: "empty" },
): void {
  event.preventDefault();
  event.stopPropagation();
  const menu: BookmarkContextMenuState = {
    ...target,
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - 236)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - 180)),
  };
  bookmarkContextMenuState.set(menu);

  refreshBookmarkContextMenu();
}

/** Right-click on a tab: offers to send it to another browser window. */
export async function showTabContextMenu(event: MouseEvent, tabId: string): Promise<void> {
  event.preventDefault();
  event.stopPropagation();
  const own = currentWindowId();
  const others = (await getBrowserWindows()).map((item) => item.id).filter((id) => id !== own);
  bookmarkContextMenuState.set({
    kind: "tab",
    tabId,
    windows: others,
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - 236)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - 180)),
  });
  refreshBookmarkContextMenu();
}

function refreshBookmarkContextMenu(): void {
  const menu = get(bookmarkContextMenuState);
  if (!menu) return;
  const actions = menu.kind === "tab"
    ? [
        ...menu.windows.map((id) => ({ id: `move-window:${id}`, label: tr("tabs.move_to_window", { number: windowNumber(id) }) })),
        { id: "move-window:new", label: tr("tabs.move_to_new_window") },
      ]
    : menu.kind === "bookmark"
    ? [
        { id: "edit-bookmark", label: tr("common.edit_bookmark") },
        { id: "delete-bookmark", label: tr("common.delete_bookmark"), danger: true },
      ]
    : menu.kind === "folder"
      ? [
          { id: "rename-folder", label: tr("common.rename_folder") },
          { id: "delete-folder", label: tr("common.delete_folder_action"), danger: true },
        ]
      : [
          { id: "create-folder", label: tr("common.create_folder") },
          { id: "manage-folders", label: tr("common.manage_folders") },
        ];
  void setBookmarkContextMenuOverlay(
    true,
    menu.x,
    menu.y,
    actions,
    get(overlayBlurPx),
  );
}

export function closeBookmarkContextMenu(): void {
  bookmarkContextMenuState.set(null);
  void setBookmarkContextMenuOverlay(false);
}

export async function closeBookmarkEditor(): Promise<void> {
  bookmarkEditorState.set(null);
  await setOverlayVisibility(false, "bookmark-editor");
}

export async function saveBookmark(
  bookmarkId: number | null,
  title: string,
  url: string,
  icon: string,
  folderId: number | null,
): Promise<void> {
  if (!isTauriRuntime()) {
    bookmarks.update((items) => {
      const existing = items.find((item) => item.id === bookmarkId || item.url === url);
      if (existing) {
        return items.map((item) => item.id === existing.id
          ? { ...item, title, url, icon, folder_id: folderId }
          : item);
      }
      return [...items, { id: Date.now(), title, url, icon, folder_id: folderId }];
    });
  } else {
    applyBookmarkData(await saveBookmarkItem(bookmarkId, title, url, icon, folderId));
  }
  await closeBookmarkEditor();
}

export async function removeBookmark(bookmarkId: number): Promise<void> {
  if (!isTauriRuntime()) {
    bookmarks.update((items) => items.filter((item) => item.id !== bookmarkId));
  } else {
    applyBookmarkData(await removeBookmarkItem(bookmarkId));
  }
  await closeBookmarkEditor();
}

export async function createBookmarkFolder(name: string): Promise<void> {
  if (!isTauriRuntime()) {
    const normalized = name.trim();
    if (!normalized) throw new Error("Folder name cannot be empty");
    bookmarkFolders.update((items) => [...items, { id: Date.now(), name: normalized }]);
    return;
  }
  applyBookmarkData(await createBookmarkFolderCommand(name));
}

export async function renameBookmarkFolder(folderId: number, name: string): Promise<void> {
  if (!isTauriRuntime()) {
    const normalized = name.trim();
    if (!normalized) throw new Error("Folder name cannot be empty");
    bookmarkFolders.update((items) => items.map((item) => item.id === folderId
      ? { ...item, name: normalized }
      : item));
    return;
  }
  applyBookmarkData(await renameBookmarkFolderCommand(folderId, name));
}

export async function deleteBookmarkFolder(folderId: number): Promise<void> {
  if (!isTauriRuntime()) {
    bookmarkFolders.update((items) => items.filter((item) => item.id !== folderId));
    bookmarks.update((items) => items.map((item) => item.folder_id === folderId
      ? { ...item, folder_id: null }
      : item));
    return;
  }
  applyBookmarkData(await deleteBookmarkFolderCommand(folderId));
}

export async function loadHistory(query = historyQuery, more = false): Promise<void> {
  if (more && (get(historyBusy) || !get(historyHasMore))) return;
  const revision = ++historyRevision;
  const current = get(historyEntries);
  const before = more ? current.at(-1) : undefined;
  historyQuery = query;
  historyBusy.set(true);
  historyError.set("");
  try {
    const list = await fetchHistory(61, query, before);
    if (revision !== historyRevision) return;
    historyHasMore.set(list.length > 60);
    historyEntries.set(more ? [...current, ...list.slice(0, 60)] : list.slice(0, 60));
  } catch (error) {
    if (revision === historyRevision) historyError.set(describeError(error));
  } finally {
    if (revision === historyRevision) historyBusy.set(false);
  }
}

export async function wipeBrowsingHistory(olderThanDays: number | null = null): Promise<void> {
  ++historyRevision;
  await clearAllHistory(olderThanDays);
  if (olderThanDays) { await loadHistory(historyQuery); return; }
  ++historyRevision;
  historyEntries.set([]);
  historyHasMore.set(false);
  historyBusy.set(false);
}

export async function openHistoryDrawer(): Promise<void> {
  isHistoryDrawerOpen.set(true);
  await setOverlayVisibility(true, "history");
  await loadHistory("");
}

export async function closeHistoryDrawer(): Promise<void> {
  ++historyRevision;
  historyBusy.set(false);
  isHistoryDrawerOpen.set(false);
  await setOverlayVisibility(false, "history");
}

export async function toggleHistoryDrawer(): Promise<void> {
  if (get(isHistoryDrawerOpen)) {
    await closeHistoryDrawer();
  } else {
    await openHistoryDrawer();
  }
}

locale.subscribe(refreshBookmarkContextMenu);
