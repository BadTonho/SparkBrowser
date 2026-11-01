import { writable, get } from "svelte/store";
import { setOverlayVisibility } from "../services/tauri-bridge";
import {
  isSidebarOpen,
  sidebarWidth,
  SIDEBAR_EXPANDED_WIDTH,
  SIDEBAR_COLLAPSED_WIDTH,
} from "./browserStore";

export const isSettingsModalOpen = writable<boolean>(false);
export const isZenMode = writable<boolean>(false);
export const autoHibernateMinutes = writable<number>(10);
export const defaultSearchEngine = writable<string>("DuckDuckGo");

export type SettingsCategory =
  | "appearance_layout"
  | "search_navigation"
  | "workspaces"
  | "performance_privacy"
  | "shortcuts";

const settingsCategories: SettingsCategory[] = [
  "appearance_layout",
  "search_navigation",
  "workspaces",
  "performance_privacy",
  "shortcuts",
];

function getInitialSettingsCategory(): SettingsCategory {
  if (typeof localStorage === "undefined") return "appearance_layout";
  const saved = localStorage.getItem("veloce_settings_category");
  return settingsCategories.includes(saved as SettingsCategory)
    ? (saved as SettingsCategory)
    : "appearance_layout";
}

export const activeSettingsCategory = writable<SettingsCategory>(
  getInitialSettingsCategory()
);

export function setActiveSettingsCategory(category: SettingsCategory): void {
  activeSettingsCategory.set(category);
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("veloce_settings_category", category);
  }
}

export type TabLayout = "horizontal" | "vertical";

const initialTabLayout: TabLayout = (() => {
  if (typeof localStorage === "undefined") return "horizontal";
  const saved = localStorage.getItem("veloce_tab_layout");
  if (saved === "vertical" || saved === "sidebar") {
    if (saved === "sidebar") {
      localStorage.setItem("veloce_tab_layout", "vertical");
    }
    return "vertical";
  }
  if (
    saved === "horizontal" ||
    saved === "top" ||
    saved === "below_omnibox"
  ) {
    if (saved !== "horizontal") {
      localStorage.setItem("veloce_tab_layout", "horizontal");
    }
    return "horizontal";
  }
  return "horizontal";
})();

export const tabLayout = writable<TabLayout>(initialTabLayout);

export function setTabLayout(layout: TabLayout): void {
  tabLayout.set(layout);
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("veloce_tab_layout", layout);
  }
}

export async function openSettingsModal(): Promise<void> {
  isSettingsModalOpen.set(true);
  await setOverlayVisibility(true, "settings");
}

export async function closeSettingsModal(): Promise<void> {
  isSettingsModalOpen.set(false);
  await setOverlayVisibility(false, "settings");
}

export async function toggleSettingsModal(): Promise<void> {
  if (get(isSettingsModalOpen)) {
    await closeSettingsModal();
  } else {
    await openSettingsModal();
  }
}

export function toggleZenMode(): void {
  const nextZen = !get(isZenMode);
  isZenMode.set(nextZen);

  if (nextZen) {
    isSidebarOpen.set(false);
    sidebarWidth.set(SIDEBAR_COLLAPSED_WIDTH);
  } else {
    isSidebarOpen.set(true);
    sidebarWidth.set(SIDEBAR_EXPANDED_WIDTH);
  }
}

