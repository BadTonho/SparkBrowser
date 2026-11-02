import { invoke, isTauriRuntime } from "./runtime";
import type { CommandSearchResult } from "../../types/command-search";
import type { SavedSession } from "../../types/browser";
export const emptyCommandSearch: CommandSearchResult = { notes: [], sessions: [], notes_count: 0, sessions_count: 0 };
export const searchCommandItems = (query: string) => isTauriRuntime() && query.trim().length <= 256 ? invoke<CommandSearchResult>("search_command_items", { query }) : Promise.resolve(emptyCommandSearch);
export const fetchSavedSession = (id: number) => invoke<SavedSession>("get_saved_session", { id });
