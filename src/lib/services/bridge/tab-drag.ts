import { invoke, isTauriRuntime } from "./runtime";
import type { TabSnapshot } from "../../types/browser";
export type DockSide = "left" | "right";
export interface DockContext { workspaceId: string; activeTabId: string; splitKey: string | null }
export interface DragPresentation {
  tabId: string; side: DockSide | null; context: DockContext; label: string; locale: string;
  pointer: { x: number; y: number };
}
let revision = Date.now(), queue: Promise<void> = Promise.resolve();
export function setDragPreview(presentation: DragPresentation | null): Promise<void> {
  const current = ++revision;
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => current === revision
    ? invoke<void>("set_tab_drag_preview", { revision: current, presentation }) : undefined);
  return queue;
}
export const flushDragPreview = (): Promise<void> => queue;
export const dockTabToSplit = (tabId: string, side: DockSide, context: DockContext) => invoke<TabSnapshot>("dock_tab_to_split", { tabId, side, context });
