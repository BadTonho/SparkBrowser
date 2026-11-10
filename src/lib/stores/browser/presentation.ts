import { writable, readonly } from "svelte/store";
import { spring } from "svelte/motion";
export const SIDEBAR_EXPANDED_WIDTH = 276;
export const SIDEBAR_COLLAPSED_WIDTH = 0;
const mutableSidebar = writable<boolean>(true);

export const isSidebarOpen = readonly(mutableSidebar);

/**
 * Spring-physics animated width for the collapsible sidebar (60/120 FPS).
 */
const animatedWidth = spring<number>(SIDEBAR_EXPANDED_WIDTH, {
  stiffness: 0.14,
  damping: 0.72,
});


export const sidebarWidth = readonly(animatedWidth);
export function setSidebarOpen(open: boolean): void {
  mutableSidebar.set(open);
  animatedWidth.set(open ? SIDEBAR_EXPANDED_WIDTH : SIDEBAR_COLLAPSED_WIDTH);
}
