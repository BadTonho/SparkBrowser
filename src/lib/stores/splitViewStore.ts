import { writable } from "svelte/store";
import type { SplitLayout } from "../types/browser";
export const splitLayout = writable<SplitLayout | null>(null);
export const splitDragRatio = writable<number | null>(null);
export const viewportBounds = writable<{ x: number; y: number; width: number; height: number } | null>(null);
export const splitGroupsKey = (layout: SplitLayout) => JSON.stringify([layout.left_tab_ids, layout.right_tab_ids]);
export const splitKey = (layout: SplitLayout) => `${layout.workspace_id}:${layout.left_tab_id}:${layout.right_tab_id}`;
