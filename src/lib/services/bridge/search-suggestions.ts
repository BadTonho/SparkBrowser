import { invoke, isTauriRuntime } from "./runtime";

/** Terms the search engine proposes for what is typed. Resolves to nothing when the engine has none or the request fails. */
export async function fetchSearchSuggestions(engine: string, query: string): Promise<string[]> {
  if (!isTauriRuntime()) return [];
  try {
    return await invoke<string[]>("fetch_search_suggestions", { engine, query });
  } catch {
    return [];
  }
}
