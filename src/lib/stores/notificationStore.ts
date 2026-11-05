import { writable } from "svelte/store";

export interface ToastNotice {
  id: string;
  message: string;
  type: "success" | "error" | "info";
  actionLabel?: string;
  onAction?: () => void;
  timeoutMs?: number;
}

export const activeToasts = writable<ToastNotice[]>([]);

export function showToast(notice: Omit<ToastNotice, "id">): string {
  const id = `toast-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
  const item: ToastNotice = {
    ...notice,
    id,
    timeoutMs: notice.timeoutMs ?? 4500,
  };

  activeToasts.update((list) => [...list, item]);

  if (item.timeoutMs && item.timeoutMs > 0) {
    setTimeout(() => {
      dismissToast(id);
    }, item.timeoutMs);
  }

  return id;
}

export function dismissToast(id: string): void {
  activeToasts.update((list) => list.filter((t) => t.id !== id));
}

