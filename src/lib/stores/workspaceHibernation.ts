import { get } from "svelte/store";
import { hibernateOtherWorkspaceTabs } from "../services/bridge/navigation";
import { isTauriRuntime } from "../services/bridge/runtime";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { hibernateOtherWorkspaces } from "./settingsStore";
import { applyTabsSnapshot } from "./browserStore";
import { logIgnored } from "../services/ignored-error";

/** Long enough that flipping through workspaces (Alt+1 … Alt+9) does not put each one to sleep on the way. */
export const WORKSPACE_HIBERNATION_DELAY_MS = 3_000;

/**
 * A moment after the shown workspace changes, and when the option is turned on, the tabs of the other
 * workspaces are put to sleep. Protected tabs (media, edited forms, downloads, pins) and exempt domains stay awake.
 */
export function initializeWorkspaceHibernation(): () => void {
  if (!isTauriRuntime()) return () => {};
  let timer: ReturnType<typeof setTimeout> | undefined;
  let disposed = false;
  const schedule = () => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (disposed || !get(hibernateOtherWorkspaces)) return;
      void hibernateOtherWorkspaceTabs().then(snapshot => { if (!disposed) applyTabsSnapshot(snapshot); }).catch(logIgnored);
    }, WORKSPACE_HIBERNATION_DELAY_MS);
  };
  let workspace = get(activeWorkspaceId);
  let enabled = get(hibernateOtherWorkspaces);
  const stopWorkspace = activeWorkspaceId.subscribe(id => {
    if (id === workspace) return;
    workspace = id;
    if (enabled) schedule();
  });
  const stopOption = hibernateOtherWorkspaces.subscribe(value => {
    const turnedOn = value && !enabled;
    enabled = value;
    if (turnedOn) schedule();
    if (!value) clearTimeout(timer);
  });
  return () => { disposed = true; clearTimeout(timer); stopWorkspace(); stopOption(); };
}
