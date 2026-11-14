import type { BookmarkData, PageNote, SavedSession, SiteSearchShortcut, SplitLayout, TabSnapshot, WorkspaceItem } from "./browser";
import type { LanguagePreference } from "../i18n";
import type { ShortcutMap } from "../stores/shortcutsStore";
import type { AutoHibernateMode, TabLayout, WorkspaceSwitcherDisplay } from "../stores/settingsStore";
import type { UpdateChannel } from "./updates";
export interface BackupPreferences {
  auto_skip_ads_enabled: boolean;
  pip_window_shadow?: boolean;
  workspace_isolation?: boolean;
  tab_layout: TabLayout; show_bookmarks_bar: boolean; overlay_blur_px: number;
  workspace_switcher_display: WorkspaceSwitcherDisplay; default_search_engine: "google" | "duckduckgo";
  auto_hibernate_mode: AutoHibernateMode; resident_limit: number; language: LanguagePreference;
  hibernation_exemptions?: string[]; ask_download_destination?: boolean;
  site_zoom?: import("./zoom").SiteZoom[];
  keyboard_shortcuts: ShortcutMap; update_channel: UpdateChannel;
}
export interface BackupTab { id: string; title: string; url: string; workspace_id: string; is_pinned: boolean; is_active: boolean; }
export interface BackupDocument {
  format: "veloce-backup"; version: 1 | 2 | 3 | 4 | 5; created_at: number;
  data: { bookmarks: BookmarkData; workspaces: WorkspaceItem[]; sessions: SavedSession[]; notes: PageNote[]; site_search: SiteSearchShortcut[]; tabs: BackupTab[]; split_layouts: SplitLayout[] };
  preferences: BackupPreferences;
}
export type BackupKind = "workspaces" | "folders" | "bookmarks" | "sessions" | "notes" | "site_search" | "tabs" | "split_layouts";
export interface BackupConflict { kind: BackupKind; key: string; reason: "existing" | "identical"; }
/** What a restore touches. Workspaces own their tabs/splits and bookmarks own their folders. */
export type RestoreGroup = "workspaces" | "bookmarks" | "sessions" | "notes" | "site_search" | "settings" | "shortcuts";
export type RestoreMode = "merge" | "replace";
export interface RestoreOptions { groups: RestoreGroup[]; mode: RestoreMode; }
export interface BackupPreview {
  token: string; created_at: number; version: number; mode: RestoreMode; groups: RestoreGroup[];
  counts: Record<BackupKind, number>; added: Record<BackupKind, number>;
  /** Local rows replace mode deletes; kinds with nothing to delete are absent. */
  removed: Partial<Record<BackupKind, number>>;
  skipped: BackupConflict[]; preferences: BackupPreferences;
}
export interface BackupPreferenceTask { id: string; preferences: BackupPreferences; error: string | null; }
export interface BackupImportResult { added: Record<BackupKind, number>; removed: Partial<Record<BackupKind, number>>; replaced: RestoreGroup[]; safety_snapshot: string | null; skipped: BackupConflict[]; pending_preferences: BackupPreferenceTask | null; snapshot: TabSnapshot; workspaces: WorkspaceItem[]; }

export type SyncTargetConfig =
  | { mode: "disabled" }
  | { mode: "google_drive_web"; auto_save: boolean; apply_preferences_on_direct_pull: boolean }
  | { mode: "folder"; directory_path: string; auto_save: boolean; apply_preferences_on_direct_pull: boolean }
  | { mode: "direct_webdav"; endpoint_url: string; username: string; remember_secret: boolean; auto_save: boolean; apply_preferences_on_direct_pull: boolean };

export interface SyncStateSnapshot {
  config: SyncTargetConfig;
  has_secret: boolean;
  connected_account?: string | null;
  last_synced_at: number | null;
  last_error: string | null;
}

export interface SyncPushResult {
  synced: boolean;
  skipped_unchanged: boolean;
  last_synced_at: number | null;
  state: SyncStateSnapshot;
}

export type SyncPullResponse =
  | { kind: "preview"; source_token: string; preview: BackupPreview; state: SyncStateSnapshot }
  | { kind: "imported"; result: BackupImportResult; state: SyncStateSnapshot };

export type LocalBackupReason = "manual" | "scheduled" | "exit" | "pre_restore";
export type LocalBackupInterval = "six_hours" | "daily" | "weekly";
export interface LocalBackupSchedule {
  enabled: boolean;
  interval: LocalBackupInterval;
  on_exit: boolean;
  max_backups: number;
  /** Backups older than this many days are deleted; null keeps them until the count limit. */
  max_age_days: number | null;
}
export interface LocalBackupEntry {
  id: string;
  created_at: number;
  reason: LocalBackupReason;
  size_bytes: number;
  encrypted: boolean;
}
export interface LocalBackupState {
  schedule: LocalBackupSchedule;
  encryption_enabled: boolean;
  last_run_at: number | null;
  next_due_at: number | null;
  directory: string;
  entries: LocalBackupEntry[];
  total_bytes: number;
}
export interface LocalBackupOutcome { created: boolean; snapshot_id: string | null; state: LocalBackupState; }
