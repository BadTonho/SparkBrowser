import { get, writable } from "svelte/store";
import { fetchSiteIcons, onSiteIconsChanged } from "../services/bridge/site-icons";

/**
 * Site icons by site key. The native side saves each page's real icon as it loads (see the favicon
 * capture) and this store only reads that cache, so a tab shows the site's own icon instead of a glyph.
 */
export const siteIcons = writable<Record<string, string>>({});

/** Mirrors the native `site_icons_repo::site_key`, so both sides group pages the same way. */
export function siteKey(raw: string): string | null {
  let url: URL;
  try { url = new URL(raw); } catch { return null; }
  if (!["http:", "https:"].includes(url.protocol) || url.username || url.password) return null;
  const host = url.hostname.replace(/^www\./, "");
  if (!host || host === "tauri.localhost") return null;
  return url.port ? `${host}:${url.port}` : host;
}

// key -> a page address with that key; a key is only asked for again after the native side reports a change.
const known = new Map<string, string>();
const asked = new Set<string>();
let listening = false;
let scheduled = false;
const pending = new Set<string>();

async function load(urls: string[]) {
  if (!urls.length) return;
  try {
    const found = await fetchSiteIcons(urls);
    const next = { ...get(siteIcons) };
    let changed = false;
    for (const [url, icon] of Object.entries(found)) {
      const key = siteKey(url);
      if (key && next[key] !== icon) { next[key] = icon; changed = true; }
    }
    if (changed) siteIcons.set(next);
  } catch { /* the generic glyph stays */ }
}

function flush() {
  scheduled = false;
  const urls = [...pending];
  pending.clear();
  void load(urls);
}

/** Asks for the icon of a page if it has not been asked for yet. Requests from one render are batched. */
export function ensureSiteIcon(url: string) {
  const key = siteKey(url);
  if (!key) return;
  if (!listening) {
    listening = true;
    void onSiteIconsChanged(() => {
      asked.clear();
      for (const sample of known.values()) { const k = siteKey(sample); if (k) { asked.add(k); pending.add(sample); } }
      if (!scheduled) { scheduled = true; queueMicrotask(flush); }
    });
  }
  known.set(key, url);
  if (asked.has(key)) return;
  asked.add(key);
  pending.add(url);
  if (!scheduled) { scheduled = true; queueMicrotask(flush); }
}
