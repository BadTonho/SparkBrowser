import { derived } from "svelte/store";
import { t, type MessageKey } from "./index";
import { errorDetails } from "../services/user-errors.js";
const messages: Record<string, MessageKey> = {
  "storage.failed": "common.storage_failed",
  "desktop.required": "common.desktop_required",
  "notes.page_required": "common.notes_page_required",
  "notes.flush_failed": "common.notes_flush_failed",
  "shortcut.modifier": "common.shortcut_modifier",
  "shortcut.number": "common.shortcut_number",
  "shortcut.conflict": "common.shortcut_conflict",
  "shortcut.invalid": "common.shortcut_invalid",
};
export const errorText = derived(t, translate => (error: unknown) => {
  const { code } = errorDetails(error);
  return translate(messages[code] ?? "common.operation_failed");
});
export const technicalError = (error: unknown) => errorDetails(error).detail;
