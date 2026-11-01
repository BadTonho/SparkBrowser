import { writable } from "svelte/store";
import { spring } from "svelte/motion";
import { setOverlayVisibility } from "../services/tauri-bridge";

export const isCommandBarOpen = writable<boolean>(false);
export const commandQuery = writable<string>("");

/**
 * Spring-physics scale for the floating Command Bar modal.
 */
export const commandBarScale = spring<number>(0.92, {
  stiffness: 0.22,
  damping: 0.68,
});

export async function openCommandBar(initialQuery = ""): Promise<void> {
  commandQuery.set(initialQuery);
  isCommandBarOpen.set(true);
  commandBarScale.set(1);
  await setOverlayVisibility(true, "command-bar");
}

export async function closeCommandBar(): Promise<void> {
  commandBarScale.set(0.92);
  isCommandBarOpen.set(false);
  commandQuery.set("");
  await setOverlayVisibility(false, "command-bar");
}

export async function toggleCommandBar(): Promise<void> {
  let currentlyOpen = false;
  isCommandBarOpen.subscribe((val) => {
    currentlyOpen = val;
  })();

  if (currentlyOpen) {
    await closeCommandBar();
  } else {
    await openCommandBar();
  }
}

