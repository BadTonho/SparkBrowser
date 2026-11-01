import { writable, get } from "svelte/store";
import type { BookmarkItem, HistoryEntry } from "../types/browser";
import {
  fetchBookmarks,
  fetchHistory,
  clearAllHistory,
  toggleBookmarkItem,
  setOverlayVisibility,
  isTauriRuntime,
} from "../services/tauri-bridge";

export const bookmarks = writable<BookmarkItem[]>([]);
export const historyEntries = writable<HistoryEntry[]>([]);
export const isHistoryDrawerOpen = writable<boolean>(false);

export async function loadBookmarks(): Promise<void> {
  const list = await fetchBookmarks();
  bookmarks.set(list);
}

export async function loadHistory(): Promise<void> {
  const list = await fetchHistory(60);
  historyEntries.set(list);
}

export async function toggleBookmarkForUrl(
  title: string,
  url: string,
  icon = "⭐"
): Promise<void> {
  if (!isTauriRuntime()) {
    bookmarks.update((items) => {
      const exists = items.some((b) => b.url === url);
      if (exists) {
        return items.filter((b) => b.url !== url);
      }
      return [...items, { id: Date.now(), title, url, icon }];
    });
    return;
  }
  const updated = await toggleBookmarkItem(title, url, icon);
  bookmarks.set(updated);
}

export async function wipeBrowsingHistory(): Promise<void> {
  await clearAllHistory();
  historyEntries.set([]);
}

export async function openHistoryDrawer(): Promise<void> {
  await loadHistory();
  isHistoryDrawerOpen.set(true);
  await setOverlayVisibility(true);
}

export async function closeHistoryDrawer(): Promise<void> {
  isHistoryDrawerOpen.set(false);
  await setOverlayVisibility(false);
}

export async function toggleHistoryDrawer(): Promise<void> {
  if (get(isHistoryDrawerOpen)) {
    await closeHistoryDrawer();
  } else {
    await openHistoryDrawer();
  }
}

