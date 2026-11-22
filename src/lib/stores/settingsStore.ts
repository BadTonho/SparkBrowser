import { setSidebarOpen } from "./browserStore";
import {
  createThemePreference,
  type ThemeMode,
  THEME_MODES,
} from "../services/theme-preference.js";
import {
  createPalettePreference,
  type Palette,
  PALETTES,
  PALETTE_STORAGE_KEY,
} from "../services/palette-preference.js";
import { safeStorage } from "../services/storage.js";
import {
  readVisualPreferences,
  writeVisualPreference,
} from "../services/visual-preferences.js";
import { writable, get } from "svelte/store";
import { setOverlayVisibility, setResidentLimit } from "../services/tauri-bridge";
import { setForcedDarkMode } from "../services/bridge/forced-dark";
import { setHibernationExemptions as setNativeHibernationExemptions } from "../services/bridge/navigation";
import { setAskDownloadDestination as setNativeAskDownloadDestination } from "../services/bridge/navigation";
import { normalizeHibernationDomain } from "../services/hibernation-domains.js";
import { closeSettingsQuickMenu } from "./settingsQuickPopoverStore";
import { setSettingsOverlay } from "../services/bridge/settings-overlay";
import { onOverlayOpening } from "../services/bridge/overlays";
import { isTauriRuntime } from "../services/bridge/runtime";
import type { PanelSession } from "../components/panels/panel-host";
import { fetchWorkspaceIsolation, setWorkspaceIsolation } from "../services/bridge/workspaces";
import {
  applyTabsSnapshot,
  tabs,
  addTab,
  selectTab,
  removeTab,
} from "./browserStore";
import { activeWorkspaceId } from "./workspaceSelectionStore";
import { createSettingsDisplayController, findSettingsTab } from "../services/settings-display.js";
import { logIgnored } from "../services/ignored-error";

export const isSettingsModalOpen = writable<boolean>(false);
export const settingsOverlaySession = writable<PanelSession | null>(null);
export const settingsTabSession = writable<(PanelSession & { tabId: string }) | null>(null);
export type SettingsDisplayMode = "popup" | "tab";
const SETTINGS_DISPLAY_KEY = "spark_settings_display";
export const settingsDisplayMode = writable<SettingsDisplayMode>(safeStorage.getItem(SETTINGS_DISPLAY_KEY) === "tab" ? "tab" : "popup");
if (typeof window !== "undefined") window.addEventListener("storage", event => {
  if (event.key === SETTINGS_DISPLAY_KEY) settingsDisplayMode.set(event.newValue === "tab" ? "tab" : "popup");
});
let settingsSessionId = 0;
let settingsTabSessionId = Date.now() * 1000;
let closingSettingsSession: number | null = null;
let closingSettingsTimer: ReturnType<typeof setTimeout> | undefined;
export const isZenMode = writable<boolean>(false);

const initialVisualPreferences = readVisualPreferences(safeStorage);
export const wideAddressBar = writable(initialVisualPreferences.wideAddressBar);
export const showFullUrls = writable(initialVisualPreferences.showFullUrls);
export const compactToolbar = writable(initialVisualPreferences.compactToolbar);
export const roundedContentFrame = writable(initialVisualPreferences.roundedContentFrame);
/** Darkens websites that are light; pages that are already dark are left as they are. */
export const forcedDarkPages = writable(initialVisualPreferences.forcedDarkPages);

export async function setForcedDarkPages(enabled: boolean): Promise<void> {
  const previous = get(forcedDarkPages);
  forcedDarkPages.set(enabled);
  try {
    await setForcedDarkMode(enabled);
    writeVisualPreference(safeStorage, "forcedDarkPages", enabled);
  } catch (error) {
    forcedDarkPages.set(previous);
    throw error;
  }
}

/** The native side forgets the choice on restart, so the saved value is sent again when the shell starts. */
export const syncForcedDarkPages = () => setForcedDarkMode(get(forcedDarkPages)).catch(logIgnored);

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

const AUTO_HIBERNATE_STORAGE_KEY = "spark_auto_hibernate_mode";
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
  // The resident limit follows this switch, so tell the native side right away.
  void setResidentTabLimit(get(residentTabLimit)).catch(logIgnored);

  if (typeof safeStorage === "undefined") return;
  try {
    safeStorage.setItem(AUTO_HIBERNATE_STORAGE_KEY, String(mode));
  } catch {
    // Keep the current setting when storage is unavailable.
  }
}

const HIBERNATION_EXEMPTIONS_KEY = "spark_hibernation_exemptions";
function loadHibernationExemptions(): string[] {
  try {
    const parsed = JSON.parse(safeStorage.getItem(HIBERNATION_EXEMPTIONS_KEY) ?? "[]");
    if (!Array.isArray(parsed)) return [];
    return [...new Set(parsed.filter((item): item is string => typeof item === "string")
      .map(item => normalizeHibernationDomain(item)).filter((item): item is string => !!item))].sort();
  } catch { return []; }
}
export const hibernationExemptions = writable<string[]>(loadHibernationExemptions());
export async function setHibernationExemptions(domains: string[]): Promise<void> {
  const normalized = [...new Set(domains.map(normalizeHibernationDomain).filter((item): item is string => !!item))].sort();
  const previous = get(hibernationExemptions);
  if (!safeStorage.setItem(HIBERNATION_EXEMPTIONS_KEY, JSON.stringify(normalized))) throw new Error("Could not save hibernation exceptions");
  hibernationExemptions.set(normalized);
  try {
    const snapshot = await setNativeHibernationExemptions(normalized);
    applyTabsSnapshot(snapshot);
  } catch (error) {
    safeStorage.setItem(HIBERNATION_EXEMPTIONS_KEY, JSON.stringify(previous));
    hibernationExemptions.set(previous);
    throw error;
  }
}

const ASK_DOWNLOAD_DESTINATION_KEY = "spark_ask_download_destination";
function getInitialAskDownloadDestination(): boolean {
  const saved = safeStorage.getItem(ASK_DOWNLOAD_DESTINATION_KEY);
  return saved === null ? true : saved === "true";
}
export const askDownloadDestination = writable(getInitialAskDownloadDestination());
export async function setAskDownloadDestination(enabled: boolean): Promise<void> {
  const previous = get(askDownloadDestination);
  askDownloadDestination.set(enabled);
  if (!safeStorage.setItem(ASK_DOWNLOAD_DESTINATION_KEY, String(enabled))) throw new Error("Could not save download preference");
  try { await setNativeAskDownloadDestination(enabled); }
  catch (error) { askDownloadDestination.set(previous); safeStorage.setItem(ASK_DOWNLOAD_DESTINATION_KEY, String(previous)); throw error; }
}
export {
  defaultSearchEngine,
  getSearchEngineHomeUrl,
  getSearchEngineLabel,
  SEARCH_ENGINES,
  setDefaultSearchEngine,
  type SearchEngine,
} from "./searchEngineStore";

const BOOKMARKS_BAR_STORAGE_KEY = "spark_show_bookmarks_bar";

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

const palettePref = createPalettePreference({
  storage: typeof safeStorage !== 'undefined' ? safeStorage : undefined,
  root: typeof document !== 'undefined' ? document.documentElement : undefined,
  changed: (palette) => colorPalette.set(palette),
});

export const colorPalette = writable<Palette>(palettePref.get());
// Every window of the browser shares this storage: when one changes the palette, the others follow.
if (typeof window !== 'undefined') {
  window.addEventListener('storage', (event) => {
    if (event.key === PALETTE_STORAGE_KEY || event.key === null) palettePref.sync();
  });
}
export { PALETTES };
export type { Palette };

export function setColorPalette(palette: Palette): void {
  colorPalette.set(palettePref.set(palette));
}

export const DEFAULT_OVERLAY_BLUR_PX = 16;
export const MAX_OVERLAY_BLUR_PX = 24;
const OVERLAY_BLUR_STORAGE_KEY = "spark_overlay_blur_px";

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

// "solid" is Matte, "glass" is the clear, light-catching glass look and "frosted" is Ice (a strong blur).
export type WindowSurfaceStyle = "frosted" | "glass" | "solid";
export const DEFAULT_WINDOW_SURFACE_STYLE: WindowSurfaceStyle = "solid";
const WINDOW_SURFACE_STYLE_STORAGE_KEY = "spark_window_surface_style";

export const DEFAULT_WINDOW_SURFACE_OPACITY = 70;
export const MIN_WINDOW_SURFACE_OPACITY = 30;
export const MAX_WINDOW_SURFACE_OPACITY = 100;
const WINDOW_SURFACE_OPACITY_STORAGE_KEY = "spark_window_surface_opacity";

export const DEFAULT_WINDOW_SURFACE_BLUR_PX = 20;
export const MAX_WINDOW_SURFACE_BLUR_PX = 32;
const WINDOW_SURFACE_BLUR_STORAGE_KEY = "spark_window_surface_blur_px";

function getInitialWindowSurfaceStyle(): WindowSurfaceStyle {
  if (typeof safeStorage === "undefined") return DEFAULT_WINDOW_SURFACE_STYLE;
  try {
    const saved = safeStorage.getItem(WINDOW_SURFACE_STYLE_STORAGE_KEY);
    return saved === "solid" || saved === "frosted" || saved === "glass" ? saved : DEFAULT_WINDOW_SURFACE_STYLE;
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

/**
 * The webview can keep showing the previous window background until a value that feeds it changes (moving the
 * opacity or blur slider fixed it). Nudging the blur by a hundredth of a pixel for two frames forces the repaint
 * as soon as the style is switched.
 */
function repaintWindowSurfaces(): void {
  if (typeof document === "undefined" || typeof requestAnimationFrame !== "function") return;
  document.documentElement.style.setProperty("--modal-surface-blur", `${get(windowSurfaceBlurPx) + 0.01}px`);
  requestAnimationFrame(() => requestAnimationFrame(() => applyWindowSurfaceBlurPx(get(windowSurfaceBlurPx))));
}

export function setWindowSurfaceStyle(style: WindowSurfaceStyle): void {
  const next: WindowSurfaceStyle = style === "solid" ? "solid" : style === "glass" ? "glass" : "frosted";
  windowSurfaceStyle.set(next);
  applyWindowSurfaceStyle(next);
  repaintWindowSurfaces();

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
  | "profiles"
  | "passwords"
  | "downloads"
  | "shortcuts"
  | "diagnostics"
  | "changelog";

const settingsCategories: SettingsCategory[] = [
  "data_backup",
  "updates",
  "appearance_layout",
  "search_navigation",
  "workspaces",
  "performance_privacy",
  "privacy_security",
  "profiles",
  "passwords",
  "downloads",
  "shortcuts",
  "diagnostics",
  "changelog",
];

function getInitialSettingsCategory(): SettingsCategory {
  if (typeof safeStorage === "undefined") return "appearance_layout";
  const saved = safeStorage.getItem("spark_settings_category");
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
    safeStorage.setItem("spark_settings_category", category);
  }
}

export type TabLayout = "horizontal" | "vertical";

export type WorkspaceSwitcherDisplay = "toolbar" | "popup";
const WORKSPACE_SWITCHER_DISPLAY_STORAGE_KEY = "spark_workspace_switcher_display";

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

export type WhatsAppDisplayMode = "popover" | "side_panel";
const WHATSAPP_DISPLAY_MODE_STORAGE_KEY = "spark_whatsapp_display_mode";

function getInitialWhatsAppDisplayMode(): WhatsAppDisplayMode {
  const saved = safeStorage.getItem(WHATSAPP_DISPLAY_MODE_STORAGE_KEY);
  return saved === "side_panel" ? "side_panel" : "popover";
}

export const whatsappDisplayMode = writable<WhatsAppDisplayMode>(getInitialWhatsAppDisplayMode());

export function setWhatsAppDisplayMode(mode: WhatsAppDisplayMode): void {
  const next = mode === "side_panel" ? "side_panel" : "popover";
  whatsappDisplayMode.set(next);
  safeStorage.setItem(WHATSAPP_DISPLAY_MODE_STORAGE_KEY, next);
}

const initialTabLayout: TabLayout = (() => {
  if (typeof safeStorage === "undefined") return "horizontal";
  const saved = safeStorage.getItem("spark_tab_layout");
  if (saved === "vertical" || saved === "sidebar") {
    if (saved === "sidebar") {
      safeStorage.setItem("spark_tab_layout", "vertical");
    }
    return "vertical";
  }
  if (
    saved === "horizontal" ||
    saved === "top" ||
    saved === "below_omnibox"
  ) {
    if (saved !== "horizontal") {
      safeStorage.setItem("spark_tab_layout", "horizontal");
    }
    return "horizontal";
  }
  return "horizontal";
})();

export const tabLayout = writable<TabLayout>(initialTabLayout);

export function setTabLayout(layout: TabLayout): void {
  tabLayout.set(layout);
  if (typeof safeStorage !== "undefined") {
    safeStorage.setItem("spark_tab_layout", layout);
  }
}

export function syncSettingsTabSession(tabId: string | null, force = false): void {
  if (!force && get(settingsTabSession)?.tabId === tabId) return;
  settingsTabSession.set(tabId ? { id: ++settingsTabSessionId, tabId, anchor: document.body as unknown as HTMLButtonElement } : null);
}

async function openSettingsTab(): Promise<string> {
  const workspace = get(activeWorkspaceId);
  const previousIds = new Set(get(tabs).map(tab => tab.id));
  const existing = findSettingsTab(get(tabs), workspace);
  if (existing) { await selectTab(existing.id); return existing.id; }
  await addTab("spark://settings", workspace);
  const created = get(tabs).find(tab => !previousIds.has(tab.id) && tab.url === "spark://settings");
  if (!created) throw new Error("Could not open settings tab");
  if (created.workspace_id !== workspace) {
    await removeTab(created.id);
    throw new Error("Settings workspace changed while opening");
  }
  return created.id;
}

const settingsDisplayController = createSettingsDisplayController({
  getMode: () => get(settingsDisplayMode), tabs: () => get(tabs), workspace: () => get(activeWorkspaceId),
  save: mode => safeStorage.setItem(SETTINGS_DISPLAY_KEY, mode), set: mode => settingsDisplayMode.set(mode),
  openTab: openSettingsTab, closeTab: removeTab, selectTab,
  popupOpen: () => get(isSettingsModalOpen), closePopup: () => closeSettingsModal(), openPopup: openSettingsPopup,
});
export const setSettingsDisplayMode = (mode: SettingsDisplayMode): Promise<void> => settingsDisplayController.change(mode);

export function restoreSettingsDisplayMode(mode: SettingsDisplayMode): void {
  settingsDisplayMode.set(mode);
}

export async function closeSettingsTab(): Promise<void> {
  const tab = get(settingsTabSession);
  if (tab) await removeTab(tab.tabId);
}

async function openSettingsPopup(): Promise<void> {
  await closeSettingsQuickMenu(false);
  clearTimeout(closingSettingsTimer);
  closingSettingsSession = null;
  if (isTauriRuntime() && !get(settingsOverlaySession)) settingsOverlaySession.set({ id: ++settingsSessionId, anchor: document.body as unknown as HTMLButtonElement });
  isSettingsModalOpen.set(true);
  await setOverlayVisibility(true, "settings");
}

export async function openSettingsModal(): Promise<void> {
  if (get(settingsDisplayMode) === "tab" && isTauriRuntime()) {
    await closeSettingsQuickMenu(false);
    await openSettingsTab();
  } else await openSettingsPopup();
}

export async function closeSettingsModal(immediate = false): Promise<void> {
  const session = get(settingsOverlaySession);
  const closingId = session?.id ?? (immediate ? closingSettingsSession : null);
  settingsOverlaySession.set(null);
  isSettingsModalOpen.set(false);
  if (closingId !== null && closingId !== undefined) {
    clearTimeout(closingSettingsTimer);
    closingSettingsSession = immediate ? null : closingId;
    if (!immediate) closingSettingsTimer = setTimeout(() => { if (closingSettingsSession === closingId) closingSettingsSession = null; }, 250);
    await setSettingsOverlay(false, closingId, undefined, !immediate, !immediate && !window.matchMedia("(prefers-reduced-motion: reduce)").matches);
  }
  await setOverlayVisibility(false, "settings");
}

onOverlayOpening(opening => {
  if (opening !== "settings" && closingSettingsSession !== null && !get(isSettingsModalOpen)) void closeSettingsModal(true);
});

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
    setSidebarOpen(false);
  } else {
    setSidebarOpen(true);
  }
}


const HIBERNATE_OTHER_WORKSPACES_KEY = "spark_hibernate_other_workspaces";
/** Put the tabs of the workspaces you are not in to sleep a moment after switching. On unless turned off. */
export const hibernateOtherWorkspaces = writable(safeStorage.getItem(HIBERNATE_OTHER_WORKSPACES_KEY) !== "false");
export function setHibernateOtherWorkspaces(enabled: boolean): boolean {
  hibernateOtherWorkspaces.set(enabled);
  return safeStorage.setItem(HIBERNATE_OTHER_WORKSPACES_KEY, String(enabled));
}

const savedResidentLimit = Number(safeStorage.getItem("spark_resident_limit"));
export const residentTabLimit = writable(Number.isInteger(savedResidentLimit) && savedResidentLimit >= 1 && savedResidentLimit <= 100 ? savedResidentLimit : 8);
export async function setResidentTabLimit(limit: number): Promise<void> {
  if (!Number.isInteger(limit) || limit < 1 || limit > 100) return;
  // With automatic hibernation off, the limit is remembered but nothing is put to sleep on its own.
  applyTabsSnapshot(await setResidentLimit(limit, get(autoHibernateMode) !== "off"));
  residentTabLimit.set(limit);
  safeStorage.setItem("spark_resident_limit", String(limit));
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
