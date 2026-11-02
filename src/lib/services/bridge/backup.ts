import { invoke } from "./runtime";
import type { BackupDocument, BackupImportResult, BackupPreferences, BackupPreferenceTask, BackupPreview } from "../../types/backup";
export const exportBackup = (preferences: BackupPreferences) => invoke<boolean>("export_backup", { preferences });
export const chooseBackupFile = () => invoke<BackupDocument | null>("choose_backup_file");
export const previewBackupImport = (document: BackupDocument, currentPreferences: BackupPreferences) => invoke<BackupPreview>("preview_backup_import", { document, currentPreferences });
export const commitBackupImport = (token: string, applyPreferences: boolean, currentPreferences: BackupPreferences) => invoke<BackupImportResult>("commit_backup_import", { token, applyPreferences, currentPreferences });
export const fetchPendingBackupPreferences = () => invoke<BackupPreferenceTask | null>("get_backup_pending_preferences");
export const finishBackupPreferences = (id: string, error: string | null) => invoke<void>("finish_backup_preferences", { id, error });
