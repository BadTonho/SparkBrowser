import { get, writable } from "svelte/store";
import { resolveTabSelection } from "../services/tab-selection.js";
import { activeWorkspaceId } from "./workspaceSelectionStore";

export const selectedTabIds = writable<string[]>([]);
let anchorId: string | null = null;

export function clearTabSelection(): void {
  anchorId = null;
  selectedTabIds.set([]);
}

/** Returns true when the input was used to modify the multi-selection. */
export function handleTabSelectionInput(
  event: Pick<MouseEvent | KeyboardEvent, "ctrlKey" | "metaKey" | "shiftKey" | "preventDefault" | "stopPropagation">,
  tabId: string,
  orderedIds: string[],
): boolean {
  const hasModifier = event.ctrlKey || event.metaKey || event.shiftKey;
  if (!hasModifier) {
    clearTabSelection();
    return false;
  }

  event.preventDefault();
  event.stopPropagation();
  const result = resolveTabSelection(get(selectedTabIds), anchorId, orderedIds, tabId, event);
  anchorId = result.anchorId;
  selectedTabIds.set(result.selectedIds);
  return true;
}

activeWorkspaceId.subscribe(() => clearTabSelection());
