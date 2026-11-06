import {
  createThemePreference,
  type ThemeMode,
  THEME_MODES,
} from "../services/theme-preference.js";
import { safeStorage } from "../services/storage.js";
import {
  readVisualPreferences,
  writeVisualPreference,
} from "../services/visual-preferences.js";
import { writable, get } from "svelte/store";
import { setOverlayVisibility, setResidentLimit } from "../services/tauri-bridge";
import { closeSettingsQuickMenu } from "./settingsQuickPopoverStore";
import { setSettingsOverlay } from "../services/bridge/settings-overlay";
import { isTauriRuntime } from "../services/bridge/runtime";
import type { PanelSession } from "../components/panels/panel-host";
import { fetchWorkspaceIsolation, setWorkspaceIsolation } from "../services/bridge/workspaces";
import {
  isSidebarOpen,
  sidebarWidth,
  SIDEBAR_EXPANDED_WIDTH,
  SIDEBAR_COLLAPSED_WIDTH,
  applyTabsSnapshot,
} from "./browserStore";

export const isSettingsModalOpen = writable<boolean>(false);
export const settingsOverlaySession = writable<PanelSession | null>(null);
let settingsSessionId = 0;
export const isZenMode = writable<boolean>(false);

const initialVisualPreferences = readVisualPreferences(safeStorage);
export const wideAddressBar = writable(initialVisualPreferences.wideAddressBar);
export const showFullUrls = writable(initialVisualPreferences.showFullUrls);
export const compactToolbar = writable(initialVisualPreferences.compactToolbar);
export const roundedContentFrame = writable(initialVisualPreferences.roundedContentFrame);

export function setWideAddressBar(enabled: boolean): void {
  wideAddressBar.set(enabled);
  writeVisualPreference(safeStorage, "wideAddressBar", enabled);
}

export function setShowFullUrls(enabled: boolean): void {
  showFullUrls.set(enabled);
  writeVisualPreference(safeStorage, "showFullUrls", enabled);
}

export function setCompactToolbar(enabled: boolean): void {
  compactToolbar.set(enabled);
  writeVisualPreference(safeStorage, "compactToolbar", enabled);
}

export function setRoundedContentFrame(enabled: boolean): void {
  roundedContentFrame.set(enabled);
  writeVisualPreference(safeStorage, "roundedContentFrame", enabled);
}

export type AutoHibernateMode = 5 | 10 | 30 | "auto" | "off";

const AUTO_HIBERNATE_STORAGE_KEY = "veloce_auto_hibernate_mode";
const AUTO_HIBERNATE_MODES: AutoHibernateMode[] = [5, 10, 30, "auto", "off"];

function getInitialAutoHibernateMode(): AutoHibernateMode {
  if (typeof safeStorage === "undefined") return 10;

  try {
    const saved = safeStorage.getItem(AUTO_HIBERNATE_STORAGE_KEY);
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

  if (typeof safeStorage === "undefined") return;
  try {
    safeStorage.setItem(AUTO_HIBERNATE_STORAGE_KEY, String(mode));
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
  if (typeof safeStorage === "undefined") return true;

  try {
    const saved = safeStorage.getItem(BOOKMARKS_BAR_STORAGE_KEY);
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
  if (typeof safeStorage === "undefined") return;

  try {
    safeStorage.setItem(BOOKMARKS_BAR_STORAGE_KEY, String(visible));
  } catch {
    // Keep the current visual setting even when storage is unavailable.
  }
}

export function toggleBookmarksBar(): void {
  setShowBookmarksBar(!get(showBookmarksBar));
}

export type { ThemeMode };
export { THEME_MODES };

const themePref = createThemePreference({
  storage: typeof safeStorage !== 'undefined' ? safeStorage : undefined,
  root: typeof document !== 'undefined' ? document.documentElement : undefined,
  changed: (mode) => themeMode.set(mode),
});

export const themeMode = writable<ThemeMode>(themePref.get());

export function setThemeMode(mode: ThemeMode): void {
  const applied = themePref.set(mode);
  themeMode.set(applied);
}

export const DEFAULT_OVERLAY_BLUR_PX = 16;
export const MAX_OVERLAY_BLUR_PX = 24;
const OVERLAY_BLUR_STORAGE_KEY = "veloce_overlay_blur_px";

function getInitialOverlayBlurPx(): number {
  if (typeof safeStorage === "undefined") return DEFAULT_OVERLAY_BLUR_PX;

  try {
    const saved = safeStorage.getItem(OVERLAY_BLUR_STORAGE_KEY);
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

  if (typeof safeStorage !== "undefined") {
    try {
      safeStorage.setItem(OVERLAY_BLUR_STORAGE_KEY, String(next));
    } catch {
      // Keep the current visual setting even when storage is unavailable.
    }
  }
}

export type WindowSurfaceStyle = "frosted" | "solid";
export const DEFAULT_WINDOW_SURFACE_STYLE: WindowSurfaceStyle = "solid";
const WINDOW_SURFACE_STYLE_STORAGE_KEY = "veloce_window_surface_style";

export const DEFAULT_WINDOW_SURFACE_OPACITY = 70;
export const MIN_WINDOW_SURFACE_OPACITY = 30;
export const MAX_WINDOW_SURFACE_OPACITY = 100;
const WINDOW_SURFACE_OPACITY_STORAGE_KEY = "veloce_window_surface_opacity";

export const DEFAULT_WINDOW_SURFACE_BLUR_PX = 20;
export const MAX_WINDOW_SURFACE_BLUR_PX = 32;
const WINDOW_SURFACE_BLUR_STORAGE_KEY = "veloce_window_surface_blur_px";

function getInitialWindowSurfaceStyle(): WindowSurfaceStyle {
  if (typeof safeStorage === "undefined") return DEFAULT_WINDOW_SURFACE_STYLE;
  try {
    const saved = safeStorage.getItem(WINDOW_SURFACE_STYLE_STORAGE_KEY);
    return saved === "solid" || saved === "frosted" ? saved : DEFAULT_WINDOW_SURFACE_STYLE;
  } catch {
    return DEFAULT_WINDOW_SURFACE_STYLE;
  }
}

function getInitialWindowSurfaceOpacity(): number {
  if (typeof safeStorage === "undefined") return DEFAULT_WINDOW_SURFACE_OPACITY;
  try {
    const saved = safeStorage.getItem(WINDOW_SURFACE_OPACITY_STORAGE_KEY);
    if (saved === null || saved.trim() === "") return DEFAULT_WINDOW_SURFACE_OPACITY;
    const value = Number(saved);
    return Number.isInteger(value) && value >= MIN_WINDOW_SURFACE_OPACITY && value <= MAX_WINDOW_SURFACE_OPACITY
      ? value
      : DEFAULT_WINDOW_SURFACE_OPACITY;
  } catch {
    return DEFAULT_WINDOW_SURFACE_OPACITY;
  }
}

function getInitialWindowSurfaceBlurPx(): number {
  if (typeof safeStorage === "undefined") return DEFAULT_WINDOW_SURFACE_BLUR_PX;
  try {
    const saved = safeStorage.getItem(WINDOW_SURFACE_BLUR_STORAGE_KEY);
    if (saved === null || saved.trim() === "") return DEFAULT_WINDOW_SURFACE_BLUR_PX;
    const value = Number(saved);
    return Number.isInteger(value) && value >= 0 && value <= MAX_WINDOW_SURFACE_BLUR_PX
      ? value
      : DEFAULT_WINDOW_SURFACE_BLUR_PX;
  } catch {
    return DEFAULT_WINDOW_SURFACE_BLUR_PX;
  }
}

function applyWindowSurfaceStyle(style: WindowSurfaceStyle): void {
  if (typeof document !== "undefined") {
    document.documentElement.dataset.windowStyle = style;
  }
}

function applyWindowSurfaceOpacity(opacity: number): void {
  if (typeof document !== "undefined") {
    document.documentElement.style.setProperty(
      "--modal-surface-opacity",
      (opacity / 100).toFixed(2)
    );
  }
}

function applyWindowSurfaceBlurPx(px: number): void {
  if (typeof document !== "undefined") {
    document.documentElement.style.setProperty("--modal-surface-blur", `${px}px`);
  }
}

export const windowSurfaceStyle = writable<WindowSurfaceStyle>(getInitialWindowSurfaceStyle());
applyWindowSurfaceStyle(get(windowSurfaceStyle));

export function setWindowSurfaceStyle(style: WindowSurfaceStyle): void {
  const next = style === "solid" ? "solid" : "frosted";
  windowSurfaceStyle.set(next);
  applyWindowSurfaceStyle(next);

  if (typeof safeStorage !== "undefined") {
    try {
      safeStorage.setItem(WINDOW_SURFACE_STYLE_STORAGE_KEY, next);
    } catch {
      // Keep setting active in memory
    }
  }
}

export const windowSurfaceOpacity = writable<number>(getInitialWindowSurfaceOpacity());
applyWindowSurfaceOpacity(get(windowSurfaceOpacity));

export function setWindowSurfaceOpacity(value: number): void {
  const rounded = Number.isFinite(value) ? Math.round(value) : DEFAULT_WINDOW_SURFACE_OPACITY;
  const next = Math.max(MIN_WINDOW_SURFACE_OPACITY, Math.min(MAX_WINDOW_SURFACE_OPACITY, rounded));
  windowSurfaceOpacity.set(next);
  applyWindowSurfaceOpacity(next);

  if (typeof safeStorage !== "undefined") {
    try {
      safeStorage.setItem(WINDOW_SURFACE_OPACITY_STORAGE_KEY, String(next));
    } catch {
      // Keep setting active in memory
    }
  }
}

export const windowSurfaceBlurPx = writable<number>(getInitialWindowSurfaceBlurPx());
applyWindowSurfaceBlurPx(get(windowSurfaceBlurPx));

export function setWindowSurfaceBlurPx(value: number): void {
  const rounded = Number.isFinite(value) ? Math.round(value) : DEFAULT_WINDOW_SURFACE_BLUR_PX;
  const next = Math.max(0, Math.min(MAX_WINDOW_SURFACE_BLUR_PX, rounded));
  windowSurfaceBlurPx.set(next);
  applyWindowSurfaceBlurPx(next);

  if (typeof safeStorage !== "undefined") {
    try {
      safeStorage.setItem(WINDOW_SURFACE_BLUR_STORAGE_KEY, String(next));
    } catch {
      // Keep setting active in memory
    }
  }
}

export type SettingsCategory =
  | "data_backup"
  | "updates"
  | "appearance_layout"
  | "search_navigation"
  | "workspaces"
  | "performance_privacy"
  | "privacy_security"
  | "shortcuts"
  | "diagnostics";

const settingsCategories: SettingsCategory[] = [
  "data_backup",
  "updates",
  "appearance_layout",
  "search_navigation",
  "workspaces",
  "performance_privacy",
  "privacy_security",
  "shortcuts",
  "diagnostics",
];

function getInitialSettingsCategory(): SettingsCategory {
  if (typeof safeStorage === "undefined") return "appearance_layout";
  const saved = safeStorage.getItem("veloce_settings_category");
  return settingsCategories.includes(saved as SettingsCategory)
    ? (saved as SettingsCategory)
    : "appearance_layout";
}

export const activeSettingsCategory = writable<SettingsCategory>(
  getInitialSettingsCategory()
);

export function setActiveSettingsCategory(category: SettingsCategory): void {
  activeSettingsCategory.set(category);
  if (typeof safeStorage !== "undefined") {
    safeStorage.setItem("veloce_settings_category", category);
  }
}

export type TabLayout = "horizontal" | "vertical";

export type WorkspaceSwitcherDisplay = "toolbar" | "popup";
const WORKSPACE_SWITCHER_DISPLAY_STORAGE_KEY = "veloce_workspace_switcher_display";

function getInitialWorkspaceSwitcherDisplay(): WorkspaceSwitcherDisplay {
  if (typeof safeStorage === "undefined") return "toolbar";

  try {
    const saved = safeStorage.getItem(WORKSPACE_SWITCHER_DISPLAY_STORAGE_KEY);
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
  if (typeof safeStorage === "undefined") return;

  try {
    safeStorage.setItem(WORKSPACE_SWITCHER_DISPLAY_STORAGE_KEY, display);
  } catch {
    // Keep the current visual setting even when storage is unavailable.
  }
}

const initialTabLayout: TabLayout = (() => {
  if (typeof safeStorage === "undefined") return "horizontal";
  const saved = safeStorage.getItem("veloce_tab_layout");
  if (saved === "vertical" || saved === "sidebar") {
    if (saved === "sidebar") {
      safeStorage.setItem("veloce_tab_layout", "vertical");
    }
    return "vertical";
  }
  if (
    saved === "horizontal" ||
    saved === "top" ||
    saved === "below_omnibox"
  ) {
    if (saved !== "horizontal") {
      safeStorage.setItem("veloce_tab_layout", "horizontal");
    }
    return "horizontal";
  }
  return "horizontal";
})();

export const tabLayout = writable<TabLayout>(initialTabLayout);

export function setTabLayout(layout: TabLayout): void {
  tabLayout.set(layout);
  if (typeof safeStorage !== "undefined") {
    safeStorage.setItem("veloce_tab_layout", layout);
  }
}

export async function openSettingsModal(): Promise<void> {
  await closeSettingsQuickMenu(false);
  if (isTauriRuntime() && !get(settingsOverlaySession)) settingsOverlaySession.set({ id: ++settingsSessionId, anchor: document.body as unknown as HTMLButtonElement });
  isSettingsModalOpen.set(true);
  await setOverlayVisibility(true, "settings");
}

export async function closeSettingsModal(): Promise<void> {
  const session = get(settingsOverlaySession);
  settingsOverlaySession.set(null);
  isSettingsModalOpen.set(false);
  if (session) await setSettingsOverlay(false, session.id, undefined, true);
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


const savedResidentLimit = Number(safeStorage.getItem("veloce_resident_limit"));
export const residentTabLimit = writable(Number.isInteger(savedResidentLimit) && savedResidentLimit >= 1 && savedResidentLimit <= 100 ? savedResidentLimit : 8);
export async function setResidentTabLimit(limit: number): Promise<void> {
  if (!Number.isInteger(limit) || limit < 1 || limit > 100) return;
  applyTabsSnapshot(await setResidentLimit(limit));
  residentTabLimit.set(limit);
  safeStorage.setItem("veloce_resident_limit", String(limit));
}

/**
 * Per-workspace content isolation is durable inside the backend database, so
 * the shell hydrates it on demand and defers persistence to the command.
 */
export const workspaceIsolation = writable<boolean>(false);
export async function loadWorkspaceIsolation(): Promise<void> {
  try {
    workspaceIsolation.set(await fetchWorkspaceIsolation());
  } catch {
    // The toggle stays on the safe default until the backend reports.
  }
}
export async function setWorkspaceIsolationEnabled(enabled: boolean): Promise<void> {
  await setWorkspaceIsolation(enabled);
  workspaceIsolation.set(enabled);
}
