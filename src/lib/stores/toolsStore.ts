import { writable } from "svelte/store";
import { setOverlayVisibility } from "../services/bridge/overlays";
export type ToolsSection = "sessions" | "tabs" | "notes" | "closed";
export const isToolsOpen = writable(false);
export const toolsSection = writable<ToolsSection>("sessions");
export async function openTools(section: ToolsSection = "sessions") {
  toolsSection.set(section);
  await setOverlayVisibility(true, "tools");
  isToolsOpen.set(true);
}
export async function closeTools() {
  isToolsOpen.set(false);
  await setOverlayVisibility(false, "tools");
}
