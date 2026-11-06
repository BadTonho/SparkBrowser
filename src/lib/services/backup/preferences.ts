import { get } from "svelte/store";
import { languagePreference, setLanguage } from "../../i18n";
import { safeStorage } from "../storage.js";
import { UserError } from "../user-errors.js";
import { askDownloadDestination, autoHibernateMode, defaultSearchEngine, hibernationExemptions, overlayBlurPx, residentTabLimit, setAskDownloadDestination, setAutoHibernateMode, setDefaultSearchEngine, setHibernationExemptions, setOverlayBlurPx, setResidentTabLimit, setShowBookmarksBar, setTabLayout, setWorkspaceSwitcherDisplay, showBookmarksBar, tabLayout, workspaceSwitcherDisplay } from "../../stores/settingsStore";
import { keyboardShortcuts, restoreShortcutMap } from "../../stores/shortcutsStore";
import { setImportedUpdateChannel, updateState } from "../../stores/updatesStore";
import type { BackupPreferences } from "../../types/backup";
import {pipWindowShadow,setPipWindowShadow} from "../../stores/pipAppearanceStore";
import {setAutoSkipAds,privacyStats} from "../../stores/privacyStore";
import {AUTO_SKIP_STORAGE_KEY} from "../auto-skip-preference.js";
import { workspaceIsolation, setWorkspaceIsolationEnabled } from "../../stores/settingsStore";
import {PIP_SHADOW_STORAGE_KEY} from "../pip-shadow-preference.js";

export function capturePreferences(): BackupPreferences {
  return { auto_skip_ads_enabled:get(privacyStats).auto_skip_ads_enabled,pip_window_shadow:get(pipWindowShadow),workspace_isolation:get(workspaceIsolation),tab_layout: get(tabLayout), show_bookmarks_bar: get(showBookmarksBar), overlay_blur_px: get(overlayBlurPx), workspace_switcher_display: get(workspaceSwitcherDisplay), default_search_engine: get(defaultSearchEngine), auto_hibernate_mode: get(autoHibernateMode), resident_limit: get(residentTabLimit), hibernation_exemptions:get(hibernationExemptions), ask_download_destination:get(askDownloadDestination), language: get(languagePreference), keyboard_shortcuts: get(keyboardShortcuts), update_channel: get(updateState).channel };
}
export async function applyBackupPreferences(p: BackupPreferences): Promise<void> {
  // Verify durable writes explicitly; existing setters intentionally tolerate storage failures.
  const entries: [string, string][] = [
    ["veloce_tab_layout", p.tab_layout], ["veloce_show_bookmarks_bar", String(p.show_bookmarks_bar)], ["veloce_overlay_blur_px", String(p.overlay_blur_px)],
    ["veloce_workspace_switcher_display", p.workspace_switcher_display], ["veloce_default_search_engine", p.default_search_engine],
    ["veloce_auto_hibernate_mode", String(p.auto_hibernate_mode)], ["veloce_resident_limit", String(p.resident_limit)],
    ["veloce_hibernation_exemptions", JSON.stringify(p.hibernation_exemptions ?? [])], ["veloce_ask_download_destination", String(p.ask_download_destination ?? true)],
    ["veloce_language", p.language], ["veloce_keyboard_shortcuts", JSON.stringify(p.keyboard_shortcuts)],
  ];
  for (const [key, value] of entries) if (!safeStorage.setItem(key, value)) throw new UserError("storage.failed", "Backup preference writes remain pending");
  setTabLayout(p.tab_layout); setShowBookmarksBar(p.show_bookmarks_bar); setOverlayBlurPx(p.overlay_blur_px);
  setWorkspaceSwitcherDisplay(p.workspace_switcher_display); setDefaultSearchEngine(p.default_search_engine); setAutoHibernateMode(p.auto_hibernate_mode);
  await setHibernationExemptions(p.hibernation_exemptions ?? []); await setAskDownloadDestination(p.ask_download_destination ?? true);
  await setAutoSkipAds(p.auto_skip_ads_enabled??false);
  if (safeStorage.getItem(AUTO_SKIP_STORAGE_KEY)!==String(p.auto_skip_ads_enabled??false)) throw new UserError("storage.failed", "Auto skip preference remains pending");
  await setPipWindowShadow(p.pip_window_shadow??false);
  if (safeStorage.getItem(PIP_SHADOW_STORAGE_KEY)!==String(p.pip_window_shadow??false)) throw new UserError("storage.failed", "Floating window preference remains pending");
  await setWorkspaceIsolationEnabled(p.workspace_isolation??false);
  if (!setLanguage(p.language)) throw new UserError("storage.failed", "Language preference remains pending");
  await restoreShortcutMap(p.keyboard_shortcuts);
  await setResidentTabLimit(p.resident_limit);
  await setImportedUpdateChannel(p.update_channel);
}
