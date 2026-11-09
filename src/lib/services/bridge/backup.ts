import { invoke } from "./runtime";
import type { BackupImportResult, BackupPreferences, BackupPreferenceTask, BackupPreview, SyncPullResponse, SyncPushResult, SyncStateSnapshot, SyncTargetConfig } from "../../types/backup";
export const exportBackup = (preferences: BackupPreferences) => invoke<boolean>("export_backup", { preferences });
export const chooseBackupFile = () => invoke<string | null>("choose_backup_file");
export const previewBackupImport = (sourceToken: string, currentPreferences: BackupPreferences) => invoke<BackupPreview>("preview_backup_import", { sourceToken, currentPreferences });
export const commitBackupImport = (token: string, applyPreferences: boolean, currentPreferences: BackupPreferences) => invoke<BackupImportResult>("commit_backup_import", { token, applyPreferences, currentPreferences });
export const fetchPendingBackupPreferences = () => invoke<BackupPreferenceTask | null>("get_backup_pending_preferences");
export const finishBackupPreferences = (id: string, error: string | null) => invoke<void>("finish_backup_preferences", { id, error });

export const discardBackupImport = (sourceToken: string) => invoke<void>("discard_backup_import", { sourceToken });

export const fetchBackupSyncState = () => invoke<SyncStateSnapshot>("get_backup_sync_state");
export const chooseBackupSyncFolder = (autoSave: boolean, applyPreferencesOnDirectPull: boolean) =>
  invoke<SyncStateSnapshot | null>("choose_backup_sync_folder", { autoSave, applyPreferencesOnDirectPull });
export const saveBackupSyncConfig = (config: SyncTargetConfig, secret: string | null, googleTabId: string | null = null) =>
  invoke<SyncStateSnapshot>("set_backup_sync_config", { config, secret, googleTabId });
export const pushBackupSync = (preferences: BackupPreferences, onlyIfChanged: boolean) =>
  invoke<SyncPushResult>("push_backup_sync", { preferences, onlyIfChanged });
export const pullBackupSync = (currentPreferences: BackupPreferences, directApply: boolean, applyPreferencesOverride: boolean | null = null) =>
  invoke<SyncPullResponse>("pull_backup_sync", { currentPreferences, directApply, applyPreferencesOverride });
