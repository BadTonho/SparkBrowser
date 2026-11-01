import { invoke } from "./runtime";

import type { BookmarkData, HistoryEntry } from "../../types/browser";

import { isTauriRuntime } from "./runtime";

export async function fetchHistory(limit = 50, query = "", before?: { visited_at: number; id: number }): Promise<HistoryEntry[]> {
  if(!isTauriRuntime()) return [];
  return await invoke<HistoryEntry[]>("get_history", { limit, query, before: before ?? null });
}

export async function clearAllHistory(): Promise<void> {
  if(!isTauriRuntime()) return;
  await invoke("clear_browsing_history");
}

export async function fetchBookmarkData(): Promise<BookmarkData> {
  if(!isTauriRuntime()) return { bookmarks: [], folders: [] };
  return await invoke<BookmarkData>("get_bookmark_data");
}

export async function saveBookmarkItem(
  bookmarkId: number | null,
  title: string,
  url: string,
  icon: string,
  folderId: number | null,
): Promise<BookmarkData> {
  if(!isTauriRuntime()) return { bookmarks: [], folders: [] };
  return await invoke<BookmarkData>("save_bookmark", {
    bookmarkId,
    title,
    url,
    icon,
    folderId,
  });
}

export async function removeBookmarkItem(bookmarkId: number): Promise<BookmarkData> {
  if(!isTauriRuntime()) return { bookmarks: [], folders: [] };
  return await invoke<BookmarkData>("remove_bookmark", { bookmarkId });
}

export async function createBookmarkFolder(name: string): Promise<BookmarkData> {
  if(!isTauriRuntime()) return { bookmarks: [], folders: [] };
  return await invoke<BookmarkData>("create_bookmark_folder", { name });
}

export async function renameBookmarkFolder(
  folderId: number,
  name: string,
): Promise<BookmarkData> {
  if(!isTauriRuntime()) return { bookmarks: [], folders: [] };
  return await invoke<BookmarkData>("rename_bookmark_folder", { folderId, name });
}

export async function deleteBookmarkFolder(folderId: number): Promise<BookmarkData> {
  if(!isTauriRuntime()) return { bookmarks: [], folders: [] };
  return await invoke<BookmarkData>("delete_bookmark_folder", { folderId });
}
