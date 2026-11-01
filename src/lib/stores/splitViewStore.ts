import { writable } from "svelte/store";
import type { SplitLayout } from "../types/browser";
export const splitLayout = writable<SplitLayout | null>(null);
export const splitDragRatio = writable<number | null>(null);
export const splitKey = (layout: SplitLayout) => `${layout.workspace_id}:${layout.left_tab_id}:${layout.right_tab_id}`;
