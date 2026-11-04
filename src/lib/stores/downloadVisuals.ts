import { fetchDownloadVisual } from "../services/bridge/downloads";
import { createDownloadVisualCache } from "../services/download-visual";
import type { DownloadVisualSource } from "../types/browser";

let cache = createDownloadVisualCache(fetchDownloadVisual);
export function resetDownloadVisuals() { cache = createDownloadVisualCache(fetchDownloadVisual); }
export const loadDownloadVisual = (source: DownloadVisualSource, stamp: string) => cache.get(source, stamp);
