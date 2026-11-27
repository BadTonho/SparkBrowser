import { invoke, isTauriRuntime } from "./runtime";
import type { BackupPreferences, GitHubBackupState, GitHubPushResult, SyncPullResponse, SyncVersion } from "../../types/backup";

export const fetchGithubBackupState = () => invoke<GitHubBackupState>("get_github_backup_state");
export const connectGithubBackup = (repository: string, branch: string, token: string | null, autoSave: boolean) =>
  invoke<GitHubBackupState>("connect_github_backup", { repository, branch, token, autoSave });
export const setGithubBackupAuto = (autoSave: boolean) => invoke<GitHubBackupState>("set_github_backup_auto", { autoSave });
export const disconnectGithubBackup = () => invoke<GitHubBackupState>("disconnect_github_backup");
export const pushGithubBackup = (preferences: BackupPreferences, onlyIfChanged: boolean) =>
  invoke<GitHubPushResult>("push_github_backup", { preferences, onlyIfChanged });
export const reviewGithubBackup = (currentPreferences: BackupPreferences) =>
  invoke<SyncPullResponse>("review_github_backup", { currentPreferences });
export const listGithubBackupVersions = () => invoke<SyncVersion[]>("list_github_backup_versions");
export const reviewGithubBackupVersion = (id: string, currentPreferences: BackupPreferences) =>
  invoke<SyncPullResponse>("review_github_backup_version", { id, currentPreferences });
export { isTauriRuntime };
