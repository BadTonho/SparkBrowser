import { writable } from "svelte/store";
import { createNoteWriter } from "../services/note-writer.js";
import { savePageNote, deletePageNote } from "../services/bridge/notes";
export const notesRevision = writable(0);
export const noteWriter = createNoteWriter({ save: savePageNote, remove: deletePageNote, changed: () => notesRevision.update(value => value + 1) });
export function noteUrl(raw: string): string {
  const url = new URL(raw);
  if (!["http:", "https:"].includes(url.protocol)) throw new Error("Notes require an HTTP/HTTPS page.");
  url.hash = ""; return url.href;
}
