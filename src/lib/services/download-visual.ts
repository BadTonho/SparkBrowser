import type { DownloadTask, DownloadVisual, DownloadVisualSource } from "../types/browser";

export const DOWNLOAD_THUMBNAIL_BUDGET = 512 * 1024;
const PNG_PREFIX = "data:image/png;base64,";
/** Only bounded, small PNG bitmaps can enter the presentation tree. */
export function isDownloadThumbnail(value: unknown): value is string {
  if (typeof value !== "string" || !value.startsWith(PNG_PREFIX) || value.length > 87_406) return false;
  const data = value.slice(PNG_PREFIX.length);
  if (data.length < 44 || data.length % 4 || !/^[A-Za-z0-9+/]+={0,2}$/.test(data)) return false;
  try {
    const header = atob(data.slice(0, 44));
    if (header.slice(0, 8) !== "\x89PNG\r\n\x1a\n" || header.slice(12, 16) !== "IHDR") return false;
    const number = (offset: number) => [0, 1, 2, 3].reduce((n, i) => n * 256 + header.charCodeAt(offset + i), 0);
    return number(8) === 13 && number(16) > 0 && number(16) <= 96 && number(20) > 0 && number(20) <= 96;
  } catch { return false; }
}

export function downloadFileKind(name: string): "image" | "video" | "audio" | "archive" | "document" | "application" | "file" {
  const ext = name.split(".").at(-1)?.toLowerCase() ?? "";
  if (["png", "jpg", "jpeg", "gif", "bmp", "webp", "avif", "svg", "ico", "tif", "tiff"].includes(ext)) return "image";
  if (["mp4", "mkv", "webm", "mov", "avi", "m4v"].includes(ext)) return "video";
  if (["mp3", "wav", "ogg", "flac", "aac", "m4a", "opus"].includes(ext)) return "audio";
  if (["zip", "rar", "7z", "tar", "gz", "bz2", "xz"].includes(ext)) return "archive";
  if (["pdf", "txt", "md", "doc", "docx", "xls", "xlsx", "csv", "ppt", "pptx", "odt", "json", "html"].includes(ext)) return "document";
  if (["exe", "msi", "msix", "appx", "dmg", "apk", "deb", "rpm"].includes(ext)) return "application";
  return "file";
}

export function downloadStatusIcon(status: string): string {
  if (["failed", "verificationfailed", "restartrequired", "interrupted"].includes(status.toLowerCase())) return "error";
  if (status.toLowerCase() === "completed") return "completed";
  if (["paused", "pausing"].includes(status.toLowerCase())) return "pause";
  return status.toLowerCase() === "cancelled" ? "close" : "processing";
}

export function formatDownloadBytes(bytes: number | null, format: (value: number) => string, unknown: string): string {
  if (bytes === null || !Number.isFinite(bytes) || bytes < 0) return unknown;
  const index = Math.max(0, Math.min(4, bytes ? Math.floor(Math.log(bytes) / Math.log(1024)) : 0));
  return `${format(Math.round(bytes / 1024 ** index * 10) / 10)} ${["B", "KB", "MB", "GB", "TB"][Math.max(0, index)]}`;
}

export function downloadTaskSize(task: DownloadTask, format: (value: number) => string, unknown: string): string {
  const received = formatDownloadBytes(task.received, format, unknown);
  if (task.status === "Completed") return received;
  if (task.total !== null && task.total > 0) return `${received} / ${formatDownloadBytes(task.total, format, unknown)}`;
  return task.received > 0 ? received : unknown;
}

/** Cache lifetime is one panel opening; at most two local thumbnail reads run together. */
export function createDownloadVisualCache(fetcher: (source: DownloadVisualSource) => Promise<DownloadVisual>, budget = DOWNLOAD_THUMBNAIL_BUDGET) {
  const cache = new Map<string, Promise<DownloadVisual>>();
  const waiting: (() => void)[] = [];
  let used = 0, running = 0;
  function next() { while (running < 2 && waiting.length) { running++; waiting.shift()!(); } }
  return {
    get(source: DownloadVisualSource, stamp: string) {
      const key = `${source.kind}:${source.id}:${stamp}`;
      if (!cache.has(key)) {
        const promise = new Promise<DownloadVisual>(resolve => {
          waiting.push(async () => {
            try {
              const value = await fetcher(source);
              const thumbnail = isDownloadThumbnail(value.thumbnail_png) && used + value.thumbnail_png.length <= budget ? value.thumbnail_png : null;
              if (thumbnail) used += thumbnail.length;
              resolve({ size_bytes: Number.isSafeInteger(value.size_bytes) && value.size_bytes! >= 0 ? value.size_bytes : null, thumbnail_png: thumbnail });
            } catch { resolve({ size_bytes: null, thumbnail_png: null }); }
            finally { running--; next(); }
          });
        });
        cache.set(key, promise);
        // Keep failures/metadata bounded too; image bytes are never returned beyond the session budget.
        while (cache.size > 256) cache.delete(cache.keys().next().value!);
        next();
      }
      return cache.get(key)!;
    },
  };
}
