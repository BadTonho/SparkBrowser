import { writable } from "svelte/store";
import { fetchSavedSession } from "../services/bridge/command-search";
import { fetchPageNote } from "../services/bridge/notes";
import { openTools } from "./toolsStore";
import { noteUrl, noteWriter, notesRevision } from "./notesStore";
import { UserError } from "../services/user-errors.js";
import type { SavedSession } from "../types/browser";
export const selectedNote = writable<{ url: string; title: string } | null>(null);
export const selectedSession = writable<SavedSession | null>(null);
export async function openSavedNote(raw: string, beforeOpen?: () => Promise<void>) {
  const url = noteUrl(raw);
  const note = await fetchPageNote(url);
  if (!note) throw new UserError("command_search.not_found", "The saved note was removed");
  await beforeOpen?.();
  noteWriter.seed(url, note.title, note.text);
  notesRevision.update(n => n + 1);
  selectedNote.set({ url, title: note.title });
  await openTools("notes");
}
export async function openSavedSession(id: number, beforeOpen?: () => Promise<void>) {
  const session = await fetchSavedSession(id);
  await beforeOpen?.();
  selectedSession.set(session);
  await openTools("sessions");
}
