import type { BookmarkData, PageNote, SavedSession, SiteSearchShortcut, SplitLayout, TabSnapshot, WorkspaceItem } from "./browser";
import type { LanguagePreference } from "../i18n";
import type { ShortcutMap } from "../stores/shortcutsStore";
import type { AutoHibernateMode, TabLayout, WorkspaceSwitcherDisplay, SettingsDisplayMode } from "../stores/settingsStore";
import type { UpdateChannel } from "./updates";
import type { SearchEngine } from "../stores/searchEngineStore";
export interface BackupPreferences {
  auto_skip_ads_enabled: boolean;
  pip_window_shadow?: boolean;
  workspace_isolation?: boolean;
  settings_display?: SettingsDisplayMode;
  tab_layout: TabLayout; show_bookmarks_bar: boolean; overlay_blur_px: number;
  workspace_switcher_display: WorkspaceSwitcherDisplay; default_search_engine: SearchEngine;
  auto_hibernate_mode: AutoHibernateMode; resident_limit: number; language: LanguagePreference;
  hibernation_exemptions?: string[]; ask_download_destination?: boolean;
  site_zoom?: import("./zoom").SiteZoom[];
  keyboard_shortcuts: ShortcutMap; update_channel: UpdateChannel;
}
export interface BackupTab { id: string; title: string; url: string; workspace_id: string; is_pinned: boolean; is_active: boolean; }
export interface BackupDocument {
  format: "spark-backup" | "veloce-backup"; version: 1 | 2 | 3 | 4 | 5 | 6; created_at: number;
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

export type BookmarkExportFormat = "html" | "csv";
export interface BookmarkExportResult { count: number; skipped: number; }

export type SyncTargetConfig =
  | { mode: "disabled" }
  | { mode: "google_drive_web"; auto_save: boolean; apply_preferences_on_direct_pull: boolean }
  | { mode: "folder"; directory_path: string; auto_save: boolean; apply_preferences_on_direct_pull: boolean }
  | { mode: "direct_webdav"; endpoint_url: string; username: string; remember_secret: boolean; auto_save: boolean; apply_preferences_on_direct_pull: boolean; nextcloud_server?: string };

export interface SyncStateSnapshot {
  config: SyncTargetConfig;
  has_secret: boolean;
  connected_account?: string | null;
  last_synced_at: number | null;
  last_error: string | null;
  /** How often the automatic sync runs and whether it is paused. */
  options: SyncOptions;
  /** The name of this computer. */
  this_device: string;
  /** The computer that wrote the file in the cloud, as last seen by a send or a download. */
  last_device: string | null;
}

export type SyncInterval = "auto" | "five_minutes" | "fifteen_minutes" | "on_exit";
export interface SyncOptions {
  interval: SyncInterval;
  /** Unix seconds until which the automatic sync does nothing. */
  paused_until: number | null;
  /** When Spark opens, download the file in the cloud and apply it if it is newer. */
  pull_on_start: boolean;
  /** What this computer sends and accepts. */
  groups: RestoreGroup[];
}

export interface SyncPushResult {
  synced: boolean;
  skipped_unchanged: boolean;
  last_synced_at: number | null;
  state: SyncStateSnapshot;
}

/** An earlier version of the file in the cloud. */
export interface SyncVersion {
  /** The copy's file name (folder, WebDAV) or the Drive revision id. */
  id: string;
  /** Unix seconds: when that version was written. */
  created_at: number;
  size: number | null;
}

/** What the check made when Spark opened. */
export type SyncStartResponse =
  | { kind: "skipped" | "up_to_date"; state: SyncStateSnapshot }
  | { kind: "applied"; result: BackupImportResult; state: SyncStateSnapshot }
  | { kind: "review"; source_token: string; preview: BackupPreview; state: SyncStateSnapshot };

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
export interface LocalBackupFailure { at: number; reason: LocalBackupReason; code: string; }
export interface LocalBackupState {
  schedule: LocalBackupSchedule;
  encryption_enabled: boolean;
  last_run_at: number | null;
  /** The latest attempt failed and no attempt has succeeded since; null otherwise. */
  last_failure: LocalBackupFailure | null;
  next_due_at: number | null;
  directory: string;
  entries: LocalBackupEntry[];
  total_bytes: number;
}
export interface LocalBackupOutcome { created: boolean; snapshot_id: string | null; state: LocalBackupState; }

/** GitHub backups: a second destination, independent from the sync target in use. */
export interface GitHubTarget { repository: string; branch: string; auto_save: boolean }
export interface GitHubBackupState { target: GitHubTarget | null; has_token: boolean; last_synced_at: number | null; last_error: string | null }
export interface GitHubPushResult { synced: boolean; skipped_unchanged: boolean; state: GitHubBackupState }
