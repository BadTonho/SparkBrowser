// Each settings section is its own chunk, read the first time its category is shown: together
// they were most of the settings window's code, and a visit usually opens one or two of them.
import type { Component } from "svelte";
import type { SettingsCategory } from "../../stores/settingsStore";

// The shortcuts section takes props and the others none, so the sections share a loose type.
type Section = Component<any>;

const loaders: Record<SettingsCategory, () => Promise<{ default: Section }>> = {
  appearance_layout: () => import("./AppearanceSettings.svelte"),
  search_navigation: () => import("./NavigationSettings.svelte"),
  workspaces: () => import("./WorkspaceSettings.svelte"),
  performance_privacy: () => import("./PerformanceSettings.svelte"),
  privacy_security: () => import("./PrivacySecuritySettings.svelte"),
  profiles: () => import("./ProfilesSettings.svelte"),
  passwords: () => import("./passwords/PasswordSettings.svelte"),
  downloads: () => import("./DownloadSettings.svelte"),
  shortcuts: () => import("./ShortcutSettings.svelte"),
  diagnostics: () => import("./DiagnosticsSettings.svelte"),
  data_backup: () => import("./BackupSettings.svelte"),
  updates: () => import("./UpdateSettings.svelte"),
  changelog: () => import("./ChangelogSettings.svelte"),
};

const loading = new Map<SettingsCategory, Promise<Section>>();

/**
 * Reads every section. The settings window that the user sees only draws what the hidden controller
 * sends, so it needs the style sheets of all sections, and reading a section loads its styles.
 */
export function preloadAllSections(): Promise<PromiseSettledResult<unknown>[]> {
  return Promise.allSettled((Object.keys(loaders) as SettingsCategory[]).map(category => loadSection(category)));
}

/** The section of `category`, read once and kept. A failed read is tried again next time. */
export function loadSection(category: SettingsCategory): Promise<Section> {
  let pending = loading.get(category);
  if (!pending) {
    pending = loaders[category]().then(module => module.default);
    pending.catch(() => loading.delete(category));
    loading.set(category, pending);
  }
  return pending;
}
