import { derived } from "svelte/store";
import { t, type MessageKey } from "./index";
import { errorDetails } from "../services/user-errors.js";
const messages: Record<string, MessageKey> = {
  "backup.invalid_url": "backup.error_invalid",
  "backup.invalid_preferences": "backup.error_invalid",
  "backup.invalid_shortcuts": "backup.error_invalid",
  "backup.invalid_name": "backup.error_invalid",
  "backup.invalid_workspace": "backup.error_invalid",
  "backup.duplicate_id": "backup.error_invalid",
  "backup.invalid_bookmark": "backup.error_invalid",
  "backup.invalid_note": "backup.error_invalid",
  "backup.invalid_session": "backup.error_invalid",
  "backup.invalid_tab": "backup.error_invalid",
  "backup.invalid_selection": "backup.error_invalid",
  "backup.invalid_split": "backup.error_invalid",
  "backup.too_many_items": "backup.error_invalid",

  "backup.preview_stale": "backup.error_stale",
  "backup.preview_missing": "backup.error_stale",
  "backup.invalid_document": "backup.error_invalid",
  "backup.unsupported_version": "backup.error_invalid",
  "backup.file_too_large": "backup.error_large",
  "backup.invalid_extension": "backup.error_extension",
  "backup.preferences_pending": "backup.error_pending",

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
