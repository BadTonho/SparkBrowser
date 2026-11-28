import { get } from "svelte/store";
import { tabs, addTab, applyTabsSnapshot } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { splitLayout } from "./splitViewStore";
import { setSplitView } from "../services/bridge/split-view";
import { pickSplitCompanion } from "../services/split-tab-groups.js";
import { logIgnored } from "../services/ignored-error";

let pending: Promise<void> | null = null;

/** One split change at a time. Pressing the shortcut or a split button again while a change is still running
 *  is ignored: otherwise each press would read the old state and open its own split (and its own blank tab). */
function exclusive(change: () => Promise<void>): Promise<void> {
  if (pending) return pending;
  pending = change()
    .catch((error) => logIgnored(error))
    .finally(() => {
      pending = null;
    });
  return pending;
}

async function openSplit(): Promise<void> {
  const workspace = get(activeWorkspaceId);
  const active = get(tabs).find((tab) => tab.is_active && (!tab.workspace_id || tab.workspace_id === workspace));
  if (!active) return;
  let right = pickSplitCompanion(get(tabs), active.id, workspace);
  if (!right) {
    await addTab(undefined, workspace);
    right = get(tabs).find((tab) => tab.is_active && tab.id !== active.id)?.id ?? null;
  }
  if (!right) throw new Error("Split view: no second tab to open on the right");
  applyTabsSnapshot(await setSplitView(active.id, right));
}

async function closeSplit(): Promise<void> {
  applyTabsSnapshot(await setSplitView(null, null));
}

/** Opens a split: the tab being viewed stays on the left and the most recently used other tab opens on the
 *  right. When it is the only tab of the workspace, a new blank tab opens on the right. */
export const enterSplitView = (): Promise<void> => exclusive(openSplit);

/** Leaves the split: every tab goes back to one list and the focused page stays selected. */
export const exitSplitView = (): Promise<void> => exclusive(closeSplit);

/** Opens or closes the split, deciding only when no other split change is running. */
export const toggleSplitView = (): Promise<void> =>
  exclusive(() => (get(splitLayout) ? closeSplit() : openSplit()));
