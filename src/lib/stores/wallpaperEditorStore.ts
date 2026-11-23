import { get } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { createSharedLifecycle } from "../services/lifecycle.js";
import { isTauriRuntime } from "../services/bridge/runtime";
import { logIgnored } from "../services/ignored-error";
import { addTab, selectTab, tabs } from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";

export const WALLPAPER_EDITOR_URL = "spark://wallpaper";

/** Opens the custom background editor in its own tab, or shows the one that is already open. */
export async function openWallpaperEditor(): Promise<void> {
  const workspace = get(activeWorkspaceId);
  const existing = get(tabs).find(tab => tab.url === WALLPAPER_EDITOR_URL && (!tab.workspace_id || tab.workspace_id === workspace));
  if (existing) await selectTab(existing.id);
  else await addTab(WALLPAPER_EDITOR_URL, workspace);
}

function start(): () => void {
  if (!isTauriRuntime()) return () => {};
  let disposed = false, stop: (() => void) | undefined;
  void listen("open-wallpaper-editor", () => { void openWallpaperEditor().catch(logIgnored); })
    .then(unlisten => { if (disposed) unlisten(); else stop = unlisten; })
    .catch(logIgnored);
  return () => { disposed = true; stop?.(); };
}

export const initializeWallpaperEditorEvents = createSharedLifecycle(scope => scope.own(start()));
