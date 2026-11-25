import { get } from "svelte/store";
import { tabs, addTab, applyTabsSnapshot } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { setSplitView } from "../services/bridge/split-view";
import { pickSplitCompanion } from "../services/split-tab-groups.js";
import { logIgnored } from "../services/ignored-error";

/** Opens a split: the tab being viewed stays on the left and the most recently used other tab opens on the
 *  right. When it is the only tab of the workspace, a new blank tab opens on the right. */
export async function enterSplitView(): Promise<void> {
  try {
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
  } catch (error) {
    logIgnored(error);
  }
}

/** Leaves the split: every tab goes back to one list and the focused page stays selected. */
export async function exitSplitView(): Promise<void> {
  try {
    applyTabsSnapshot(await setSplitView(null, null));
  } catch (error) {
    logIgnored(error);
  }
}
