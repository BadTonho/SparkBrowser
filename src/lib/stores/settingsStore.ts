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
export type AutoHibernateMode = 5 | 10 | 30 | "auto" | "off";

const AUTO_HIBERNATE_STORAGE_KEY = "veloce_auto_hibernate_mode";
const AUTO_HIBERNATE_MODES: AutoHibernateMode[] = [5, 10, 30, "auto", "off"];

function getInitialAutoHibernateMode(): AutoHibernateMode {
  if (typeof localStorage === "undefined") return 10;

  try {
    const saved = localStorage.getItem(AUTO_HIBERNATE_STORAGE_KEY);
    if (saved === "5" || saved === "10" || saved === "30") return Number(saved) as 5 | 10 | 30;
    return AUTO_HIBERNATE_MODES.includes(saved as AutoHibernateMode)
      ? (saved as AutoHibernateMode)
      : 10;
  } catch {
    return 10;
  }
}

export const autoHibernateMode = writable<AutoHibernateMode>(getInitialAutoHibernateMode());

export function setAutoHibernateMode(mode: AutoHibernateMode): void {
  if (!AUTO_HIBERNATE_MODES.includes(mode)) return;
  autoHibernateMode.set(mode);

  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(AUTO_HIBERNATE_STORAGE_KEY, String(mode));
  } catch {
    // Keep the current setting when storage is unavailable.
  }
}
export {
  defaultSearchEngine,
  getSearchEngineHomeUrl,
  getSearchEngineLabel,
  SEARCH_ENGINES,
  setDefaultSearchEngine,
  type SearchEngine,
} from "./searchEngineStore";

const BOOKMARKS_BAR_STORAGE_KEY = "veloce_show_bookmarks_bar";

function getInitialShowBookmarksBar(): boolean {
  if (typeof localStorage === "undefined") return true;

  try {
    const saved = localStorage.getItem(BOOKMARKS_BAR_STORAGE_KEY);
    if (saved === "true") return true;
    if (saved === "false") return false;
    return true;
  } catch {
    return true;
  }
}

export const showBookmarksBar = writable<boolean>(getInitialShowBookmarksBar());

export function setShowBookmarksBar(visible: boolean): void {
  showBookmarksBar.set(visible);
  if (typeof localStorage === "undefined") return;

  try {
    localStorage.setItem(BOOKMARKS_BAR_STORAGE_KEY, String(visible));
  } catch {
    // Keep the current visual setting even when storage is unavailable.
  }
}

export function toggleBookmarksBar(): void {
  setShowBookmarksBar(!get(showBookmarksBar));
}

export const DEFAULT_OVERLAY_BLUR_PX = 16;
export const MAX_OVERLAY_BLUR_PX = 24;
const OVERLAY_BLUR_STORAGE_KEY = "veloce_overlay_blur_px";

function getInitialOverlayBlurPx(): number {
  if (typeof localStorage === "undefined") return DEFAULT_OVERLAY_BLUR_PX;

  try {
    const saved = localStorage.getItem(OVERLAY_BLUR_STORAGE_KEY);
    if (saved === null || saved.trim() === "") return DEFAULT_OVERLAY_BLUR_PX;

    const value = Number(saved);
    return Number.isInteger(value) && value >= 0 && value <= MAX_OVERLAY_BLUR_PX
      ? value
      : DEFAULT_OVERLAY_BLUR_PX;
  } catch {
    return DEFAULT_OVERLAY_BLUR_PX;
  }
}

function applyOverlayBlurPx(value: number): void {
  if (typeof document !== "undefined") {
    document.documentElement.style.setProperty("--overlay-backdrop-blur", `${value}px`);
  }
}

export const overlayBlurPx = writable<number>(getInitialOverlayBlurPx());
applyOverlayBlurPx(get(overlayBlurPx));

export function setOverlayBlurPx(value: number): void {
  const rounded = Number.isFinite(value) ? Math.round(value) : DEFAULT_OVERLAY_BLUR_PX;
  const next = Math.max(0, Math.min(MAX_OVERLAY_BLUR_PX, rounded));
  overlayBlurPx.set(next);
  applyOverlayBlurPx(next);

  if (typeof localStorage !== "undefined") {
    try {
      localStorage.setItem(OVERLAY_BLUR_STORAGE_KEY, String(next));
    } catch {
      // Keep the current visual setting even when storage is unavailable.
    }
  }
}

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

export type WorkspaceSwitcherDisplay = "toolbar" | "popup";
const WORKSPACE_SWITCHER_DISPLAY_STORAGE_KEY = "veloce_workspace_switcher_display";

function getInitialWorkspaceSwitcherDisplay(): WorkspaceSwitcherDisplay {
  if (typeof localStorage === "undefined") return "toolbar";

  try {
    const saved = localStorage.getItem(WORKSPACE_SWITCHER_DISPLAY_STORAGE_KEY);
    return saved === "popup" ? "popup" : "toolbar";
  } catch {
    return "toolbar";
  }
}

export const workspaceSwitcherDisplay = writable<WorkspaceSwitcherDisplay>(
  getInitialWorkspaceSwitcherDisplay()
);

export function setWorkspaceSwitcherDisplay(display: WorkspaceSwitcherDisplay): void {
  workspaceSwitcherDisplay.set(display);
  if (typeof localStorage === "undefined") return;

  try {
    localStorage.setItem(WORKSPACE_SWITCHER_DISPLAY_STORAGE_KEY, display);
  } catch {
    // Keep the current visual setting even when storage is unavailable.
  }
}

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

