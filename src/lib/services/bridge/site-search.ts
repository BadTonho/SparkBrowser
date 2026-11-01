import { invoke, isTauriRuntime } from "./runtime";
import { safeStorage } from "../storage.js";
import { validateSiteSearch } from "../site-search.js";
import type { SiteSearchShortcut } from "../../types/browser";
const key = "veloce_site_search_shortcuts";
function preview(): SiteSearchShortcut[] {
  try { const rows = JSON.parse(safeStorage.getItem(key) ?? "[]") as SiteSearchShortcut[]; if (!Array.isArray(rows)) return []; rows.forEach(validateSiteSearch); return rows; } catch { return []; }
}
export const fetchSiteSearchShortcuts = () => isTauriRuntime() ? invoke<SiteSearchShortcut[]>("get_site_search_shortcuts") : Promise.resolve(preview());
export async function saveSiteSearchShortcut(item: SiteSearchShortcut, previous: string | null): Promise<void> {
  validateSiteSearch(item);
  if (isTauriRuntime()) return invoke("save_site_search_shortcut", { item, previous });
  const rows = preview().filter(row => row.keyword.toLowerCase() !== previous?.toLowerCase());
  if (rows.some(row => row.keyword.toLowerCase() === item.keyword.toLowerCase())) throw new Error("Keyword already exists");
  safeStorage.setItem(key, JSON.stringify([...rows, { ...item, keyword: item.keyword.toLowerCase() }]));
}
export async function deleteSiteSearchShortcut(keyword: string): Promise<void> {
  if (isTauriRuntime()) return invoke("delete_site_search_shortcut", { keyword });
  safeStorage.setItem(key, JSON.stringify(preview().filter(row => row.keyword !== keyword)));
}
