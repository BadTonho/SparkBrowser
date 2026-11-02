import { UserError } from "../user-errors.js";
import { invoke, isTauriRuntime } from "./runtime";
import type { SavedSession, TabSnapshot } from "../../types/browser";
export const fetchSavedSessions = () => isTauriRuntime() ? invoke<SavedSession[]>("get_saved_sessions") : Promise.resolve([]);
export async function saveNamedSession(name: string, id: number | null = null): Promise<void> {
  if (!isTauriRuntime()) throw new UserError("desktop.required", "Named sessions require the desktop application.");
  await invoke("save_named_session", { name, id });
}
export const renameNamedSession = (id: number, name: string) => invoke<void>("rename_named_session", { id, name });
export const deleteNamedSession = (id: number) => invoke<void>("delete_named_session", { id });
export const restoreNamedSession = (id: number, expectedSession: SavedSession | null = null) => invoke<{ snapshot: TabSnapshot; warnings: string[] }>("restore_named_session", { id, expectedSession });
