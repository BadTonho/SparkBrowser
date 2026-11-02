import { invoke } from "./runtime";
import type { BackupImportResult, BackupPreferences, BackupPreferenceTask, BackupPreview } from "../../types/backup";
export const exportBackup = (preferences: BackupPreferences) => invoke<boolean>("export_backup", { preferences });
export const chooseBackupFile = () => invoke<string | null>("choose_backup_file");
export const previewBackupImport = (sourceToken: string, currentPreferences: BackupPreferences) => invoke<BackupPreview>("preview_backup_import", { sourceToken, currentPreferences });
export const commitBackupImport = (token: string, applyPreferences: boolean, currentPreferences: BackupPreferences) => invoke<BackupImportResult>("commit_backup_import", { token, applyPreferences, currentPreferences });
export const fetchPendingBackupPreferences = () => invoke<BackupPreferenceTask | null>("get_backup_pending_preferences");
export const finishBackupPreferences = (id: string, error: string | null) => invoke<void>("finish_backup_preferences", { id, error });

export const discardBackupImport = (sourceToken: string) => invoke<void>("discard_backup_import", { sourceToken });
