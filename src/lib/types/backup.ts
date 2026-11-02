import type { BookmarkData, PageNote, SavedSession, SiteSearchShortcut, SplitLayout, TabSnapshot, WorkspaceItem } from "./browser";
import type { LanguagePreference } from "../i18n";
import type { ShortcutMap } from "../stores/shortcutsStore";
import type { AutoHibernateMode, TabLayout, WorkspaceSwitcherDisplay } from "../stores/settingsStore";
import type { UpdateChannel } from "./updates";
export interface BackupPreferences {
  tab_layout: TabLayout; show_bookmarks_bar: boolean; overlay_blur_px: number;
  workspace_switcher_display: WorkspaceSwitcherDisplay; default_search_engine: "google" | "duckduckgo";
  auto_hibernate_mode: AutoHibernateMode; resident_limit: number; language: LanguagePreference;
  keyboard_shortcuts: ShortcutMap; update_channel: UpdateChannel;
}
export interface BackupTab { id: string; title: string; url: string; workspace_id: string; is_pinned: boolean; is_active: boolean; }
export interface BackupDocument {
  format: "veloce-backup"; version: 1; created_at: number;
  data: { bookmarks: BookmarkData; workspaces: WorkspaceItem[]; sessions: SavedSession[]; notes: PageNote[]; site_search: SiteSearchShortcut[]; tabs: BackupTab[]; split_layouts: SplitLayout[] };
  preferences: BackupPreferences;
}
export type BackupKind = "workspaces" | "folders" | "bookmarks" | "sessions" | "notes" | "site_search" | "tabs" | "split_layouts";
export interface BackupConflict { kind: BackupKind; key: string; reason: "existing" | "identical"; }
export interface BackupPreview { token: string; counts: Record<BackupKind, number>; added: Record<BackupKind, number>; skipped: BackupConflict[]; preferences: BackupPreferences; }
export interface BackupPreferenceTask { id: string; preferences: BackupPreferences; error: string | null; }
export interface BackupImportResult { added: Record<BackupKind, number>; skipped: BackupConflict[]; pending_preferences: BackupPreferenceTask | null; snapshot: TabSnapshot; workspaces: WorkspaceItem[]; }
