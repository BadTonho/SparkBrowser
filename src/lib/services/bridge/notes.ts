import { UserError } from "../user-errors.js";
import { invoke, isTauriRuntime } from "./runtime";
import type { PageNote, NoteSummary } from "../../types/browser";
export const fetchPageNote = (url: string) => isTauriRuntime() ? invoke<PageNote | null>("get_page_note", { url }) : Promise.resolve(null);
export const searchPageNotes = (query: string) => isTauriRuntime() ? invoke<NoteSummary[]>("search_page_notes", { query }) : Promise.resolve([]);
export async function savePageNote(url: string, title: string, text: string): Promise<void> {
  if (!isTauriRuntime()) throw new UserError("desktop.required", "Persistent notes require the desktop application.");
  await invoke("save_page_note", { url, title, text });
}
export const deletePageNote = (url: string) => invoke<void>("delete_page_note", { url });
