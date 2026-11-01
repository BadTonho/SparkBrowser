import { writable, get } from "svelte/store";
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
  | null;

export const bookmarks = writable<BookmarkItem[]>([]);
export const bookmarkFolders = writable<BookmarkFolder[]>([]);
export const bookmarkEditorState = writable<BookmarkEditorState>(null);
export const bookmarkContextMenuState = writable<BookmarkContextMenuState>(null);
export const historyEntries = writable<HistoryEntry[]>([]);
export const isHistoryDrawerOpen = writable<boolean>(false);

function applyBookmarkData(data: BookmarkData): void {
  bookmarks.set(data.bookmarks);
  bookmarkFolders.set(data.folders);
}

export async function loadBookmarks(): Promise<void> {
  applyBookmarkData(await fetchBookmarkData());
}

export async function openBookmarkEditor(title: string, url: string, icon = "⭐"): Promise<void> {
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

  if (!menu) return;
  const actions = menu.kind === "bookmark"
    ? [
        { id: "edit-bookmark", label: "Edit bookmark" },
        { id: "delete-bookmark", label: "Delete bookmark", danger: true },
      ]
    : menu.kind === "folder"
      ? [
          { id: "rename-folder", label: "Rename folder" },
          { id: "delete-folder", label: "Delete folder", danger: true },
        ]
      : [
          { id: "create-folder", label: "Create folder" },
          { id: "manage-folders", label: "Manage folders" },
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

export async function loadHistory(): Promise<void> {
  const list = await fetchHistory(60);
  historyEntries.set(list);
}

export async function wipeBrowsingHistory(): Promise<void> {
  await clearAllHistory();
  historyEntries.set([]);
}

export async function openHistoryDrawer(): Promise<void> {
  await loadHistory();
  isHistoryDrawerOpen.set(true);
  await setOverlayVisibility(true, "history");
}

export async function closeHistoryDrawer(): Promise<void> {
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
