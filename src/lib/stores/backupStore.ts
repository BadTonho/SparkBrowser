import { applyBackupZoom } from "../services/bridge/zoom";
import { get, writable } from "svelte/store";
import { isTauriRuntime } from "../services/bridge/runtime";
import { chooseBackupFile, discardBackupImport, commitBackupImport, exportBackup, fetchPendingBackupPreferences, finishBackupPreferences, previewBackupImport } from "../services/bridge/backup";
import { applyBackupPreferences, capturePreferences } from "../services/backup/preferences";
import { describeError } from "../services/user-errors.js";
import { noteWriter } from "./notesStore";
import { applyTabsSnapshot } from "./browserStore";
import { workspaces } from "./workspaceStore";
import { loadBookmarks } from "./persistenceStore";
import type { BackupPreferenceTask, BackupPreview, BackupImportResult } from "../types/backup";

export const backupBusy = writable(false);
export const backupError = writable("");
export const backupPreview = writable<BackupPreview | null>(null);
export const backupResult = writable<BackupImportResult | null>(null);
export const backupExported = writable(false);
export const pendingBackupPreferences = writable<BackupPreferenceTask | null>(null);
let source: string | null = null;
async function action(work: () => Promise<void>): Promise<void> {
  if (get(backupBusy) || !isTauriRuntime()) return;
  backupBusy.set(true); backupError.set("");
  try { await work(); } catch (error) { backupError.set(describeError(error)); }
  finally { backupBusy.set(false); }
}
export const saveBackup = () => action(async () => { await noteWriter.flushAll(); backupExported.set(await exportBackup(capturePreferences())); });
export const selectBackup = () => action(async () => {
  if (source) await discardBackupImport(source);
  backupResult.set(null); backupPreview.set(null); source = null;
  const document = await chooseBackupFile(); if (!document) return;
  source = document; await createPreview();
});
async function createPreview(): Promise<void> {
  if (!source) return;
  await noteWriter.flushAll(); backupPreview.set(await previewBackupImport(source, capturePreferences()));
}
export const refreshBackupPreview = () => action(createPreview);
async function applyPending(): Promise<void> {
  const task = await fetchPendingBackupPreferences(); pendingBackupPreferences.set(task);
  if (!task) return;
  try { await applyBackupPreferences(task.preferences); await applyBackupZoom(task.id); await finishBackupPreferences(task.id, null); pendingBackupPreferences.set(null); }
  catch (error) {
    const detail = describeError(error); await finishBackupPreferences(task.id, detail);
    pendingBackupPreferences.set({ ...task, error: detail }); throw error;
  }
}
export const retryBackupPreferences = () => action(applyPending);
export const importBackup = (applyPreferences: boolean) => action(async () => {
  const preview = get(backupPreview); if (!preview) return;
  await noteWriter.flushAll();
  const result = await commitBackupImport(preview.token, applyPreferences, capturePreferences());
  // Import data is already committed, even if applying preferences or refreshing the UI fails.
  backupResult.set(result); backupPreview.set(null); source = null;
  workspaces.set(result.workspaces); applyTabsSnapshot(result.snapshot);
  pendingBackupPreferences.set(result.pending_preferences);
  await loadBookmarks();
  if (result.pending_preferences) await applyPending();
});
export function initializeBackupStatus(): void {
  if (isTauriRuntime()) void fetchPendingBackupPreferences().then(value => pendingBackupPreferences.set(value)).catch(error => backupError.set(describeError(error)));
}
export function dismissBackupPreview(): void { const token = source; source = null; backupPreview.set(null); if (token) void discardBackupImport(token).catch(error => backupError.set(describeError(error))); }
