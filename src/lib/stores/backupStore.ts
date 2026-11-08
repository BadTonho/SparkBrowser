import { applyBackupZoom } from "../services/bridge/zoom";
import { get, writable } from "svelte/store";
import { isTauriRuntime } from "../services/bridge/runtime";
import {
  chooseBackupFile,
  chooseBackupSyncFolder,
  commitBackupImport,
  discardBackupImport,
  exportBackup,
  fetchBackupSyncState,
  fetchPendingBackupPreferences,
  finishBackupPreferences,
  previewBackupImport,
  pullBackupSync,
  pushBackupSync,
  saveBackupSyncConfig,
} from "../services/bridge/backup";
import { applyBackupPreferences, capturePreferences } from "../services/backup/preferences";
import { describeError } from "../services/user-errors.js";
import { noteWriter } from "./notesStore";
import { addTab, applyTabsSnapshot, tabs } from "./browserStore";
import { reloadCurrentPage } from "../services/tauri-bridge";
import { workspaces } from "./workspaceStore";
import { bookmarkFolders, bookmarks, loadBookmarks } from "./persistenceStore";
import { splitLayout } from "./splitViewStore";
import {
  askDownloadDestination,
  autoHibernateMode,
  closeSettingsModal,
  defaultSearchEngine,
  hibernationExemptions,
  overlayBlurPx,
  residentTabLimit,
  showBookmarksBar,
  tabLayout,
  workspaceIsolation,
  workspaceSwitcherDisplay,
} from "./settingsStore";
import { languagePreference } from "../i18n";
import { keyboardShortcuts } from "./shortcutsStore";
import { pipWindowShadow } from "./pipAppearanceStore";
import type {
  BackupImportResult,
  BackupPreferenceTask,
  BackupPreview,
  SyncStateSnapshot,
  SyncTargetConfig,
} from "../types/backup";

export const backupBusy = writable(false);
export const backupError = writable("");
export const backupPreview = writable<BackupPreview | null>(null);
export const backupResult = writable<BackupImportResult | null>(null);
export const backupExported = writable(false);
export const pendingBackupPreferences = writable<BackupPreferenceTask | null>(null);
export const backupSyncState = writable<SyncStateSnapshot>({
  config: { mode: "disabled" },
  has_secret: false,
  connected_account: null,
  last_synced_at: null,
  last_error: null,
});
export const backupSyncNotice = writable<"pushed" | "unchanged" | "config_saved" | "">("");

let source: string | null = null;
let folderAutoSaveTimer: ReturnType<typeof setTimeout> | undefined;
let cloudMinuteInterval: ReturnType<typeof setInterval> | undefined;
let autoSyncUnsubscribes: (() => void)[] = [];
let hasPendingChangesForCloud = false;
let autoSyncReady = false;
let gdriveLoginRequiredPause = false;
let awaitingGoogleAccountSelection = false;
let lastTabsSignature = "";

async function action(work: () => Promise<void>): Promise<void> {
  if (get(backupBusy) || !isTauriRuntime()) return;
  backupBusy.set(true);
  backupError.set("");
  backupSyncNotice.set("");
  try {
    await work();
  } catch (error) {
    backupError.set(describeError(error));
  } finally {
    backupBusy.set(false);
  }
}

export const saveBackup = () =>
  action(async () => {
    await noteWriter.flushAll();
    backupExported.set(await exportBackup(capturePreferences()));
  });

export const selectBackup = () =>
  action(async () => {
    if (source) await discardBackupImport(source);
    backupResult.set(null);
    backupPreview.set(null);
    source = null;
    const document = await chooseBackupFile();
    if (!document) return;
    source = document;
    await createPreview();
  });

async function createPreview(): Promise<void> {
  if (!source) return;
  await noteWriter.flushAll();
  backupPreview.set(await previewBackupImport(source, capturePreferences()));
}

export const refreshBackupPreview = () => action(createPreview);

async function applyPending(): Promise<void> {
  const task = await fetchPendingBackupPreferences();
  pendingBackupPreferences.set(task);
  if (!task) return;
  try {
    await applyBackupPreferences(task.preferences);
    await applyBackupZoom(task.id);
    await finishBackupPreferences(task.id, null);
    pendingBackupPreferences.set(null);
  } catch (error) {
    const detail = describeError(error);
    await finishBackupPreferences(task.id, detail);
    pendingBackupPreferences.set({ ...task, error: detail });
    throw error;
  }
}

export const retryBackupPreferences = () => action(applyPending);

async function applyImportedResult(result: BackupImportResult): Promise<void> {
  backupResult.set(result);
  backupPreview.set(null);
  source = null;
  workspaces.set(result.workspaces);
  applyTabsSnapshot(result.snapshot);
  pendingBackupPreferences.set(result.pending_preferences);
  await loadBookmarks();
  if (result.pending_preferences) await applyPending();
}

export const importBackup = (applyPreferences: boolean) =>
  action(async () => {
    const preview = get(backupPreview);
    if (!preview) return;
    await noteWriter.flushAll();
    const result = await commitBackupImport(preview.token, applyPreferences, capturePreferences());
    // Import data is already committed, even if applying preferences or refreshing the UI fails.
    await applyImportedResult(result);
  });

export const pickBackupSyncFolder = (
  autoSave = true,
  applyPreferencesOnDirectPull = false
) =>
  action(async () => {
    const snap = await chooseBackupSyncFolder(autoSave, applyPreferencesOnDirectPull);
    if (!snap) return;
    backupSyncState.set(snap);
    backupSyncNotice.set("config_saved");
    if (autoSave) {
      await noteWriter.flushAll();
      const pushed = await pushBackupSync(capturePreferences(), true);
      backupSyncState.set(pushed.state);
      hasPendingChangesForCloud = false;
    }
  });

export const saveBackupSyncSettings = (
  config: SyncTargetConfig,
  secret: string | null = null
) =>
  action(async () => {
    const snap = await saveBackupSyncConfig(config, secret);
    backupSyncState.set(snap);
    backupSyncNotice.set("config_saved");
    if (config.mode !== "disabled" && config.auto_save) {
      hasPendingChangesForCloud = true;
      if (config.mode === "folder") {
        scheduleFolderAutoSync();
      } else if (config.mode === "google_drive_web" || config.mode === "direct_webdav") {
        gdriveLoginRequiredPause = false;
        setTimeout(() => void runBackgroundAutoSync(), 150);
      }
    }
  });

export async function openGoogleLoginTab(): Promise<void> {
  gdriveLoginRequiredPause = false;
  awaitingGoogleAccountSelection = true;
  await closeSettingsModal();
  await addTab(
    "https://accounts.google.com/AccountChooser?continue=https%3A%2F%2Fdrive.google.com%2Fdrive%2Fmy-drive&service=wise"
  );
}

export const syncPushNow = () =>
  action(async () => {
    gdriveLoginRequiredPause = false;
    await noteWriter.flushAll();
    const res = await pushBackupSync(capturePreferences(), false);
    backupSyncState.set(res.state);
    hasPendingChangesForCloud = false;
    backupSyncNotice.set(res.skipped_unchanged ? "unchanged" : "pushed");
    const activeTab = get(tabs).find((t) => t.is_active);
    if (activeTab && activeTab.url.includes("drive.google.com")) {
      void reloadCurrentPage().catch(() => {});
    }
  });

export const syncPullPreview = () =>
  action(async () => {
    if (source) await discardBackupImport(source);
    backupResult.set(null);
    backupPreview.set(null);
    source = null;
    await noteWriter.flushAll();
    const res = await pullBackupSync(capturePreferences(), false, null);
    backupSyncState.set(res.state);
    if (res.kind === "preview") {
      source = res.source_token;
      backupPreview.set(res.preview);
    }
  });

export const syncPullDirect = (applyPreferencesOverride: boolean | null = null) =>
  action(async () => {
    if (source) await discardBackupImport(source);
    backupResult.set(null);
    backupPreview.set(null);
    source = null;
    await noteWriter.flushAll();
    const res = await pullBackupSync(capturePreferences(), true, applyPreferencesOverride);
    backupSyncState.set(res.state);
    if (res.kind === "imported") {
      hasPendingChangesForCloud = false;
      await applyImportedResult(res.result);
    }
  });

async function runBackgroundAutoSync(reloadDriveTabOnSuccess = false): Promise<void> {
  if (!isTauriRuntime() || get(backupBusy)) return;
  const current = get(backupSyncState).config;
  if (current.mode === "disabled" || !current.auto_save) return;
  if (current.mode === "google_drive_web" && gdriveLoginRequiredPause) return;
  try {
    await noteWriter.flushAll();
    const res = await pushBackupSync(capturePreferences(), true);
    backupSyncState.set(res.state);
    hasPendingChangesForCloud = false;
    gdriveLoginRequiredPause = false;
    if (reloadDriveTabOnSuccess && res.synced) {
      const activeTab = get(tabs).find((t) => t.is_active);
      if (activeTab && activeTab.url.includes("drive.google.com")) {
        void reloadCurrentPage().catch(() => {});
      }
    }
  } catch (error) {
    const raw = String(error);
    if (raw.includes("backup.sync_gdrive_login_required")) {
      gdriveLoginRequiredPause = true;
    }
    backupSyncState.update((s) => ({ ...s, last_error: describeError(error) }));
  }
}

function scheduleFolderAutoSync(): void {
  if (folderAutoSaveTimer) clearTimeout(folderAutoSaveTimer);
  folderAutoSaveTimer = setTimeout(() => {
    folderAutoSaveTimer = undefined;
    const cfg = get(backupSyncState).config;
    if (cfg.mode === "folder" && cfg.auto_save) {
      void runBackgroundAutoSync();
    }
  }, 1000);
}

function notifyPersistentDataChanged(): void {
  if (!autoSyncReady) return;
  hasPendingChangesForCloud = true;
  const cfg = get(backupSyncState).config;
  if (cfg.mode === "folder" && cfg.auto_save) {
    scheduleFolderAutoSync();
  }
}

export async function flushBackupSyncBeforeShutdown(): Promise<void> {
  if (!isTauriRuntime()) return;
  if (folderAutoSaveTimer) {
    clearTimeout(folderAutoSaveTimer);
    folderAutoSaveTimer = undefined;
  }
  const cfg = get(backupSyncState).config;
  if (cfg.mode === "disabled" || !cfg.auto_save) return;
  try {
    const res = await pushBackupSync(capturePreferences(), true);
    backupSyncState.set(res.state);
    hasPendingChangesForCloud = false;
  } catch {
    // Never block browser shutdown if the external folder or remote WebDAV server is unreachable.
  }
}

export function initializeBackupStatus(): void {
  if (!isTauriRuntime()) return;
  void fetchPendingBackupPreferences()
    .then((value) => pendingBackupPreferences.set(value))
    .catch((error) => backupError.set(describeError(error)));

  void fetchBackupSyncState()
    .then((state) => {
      backupSyncState.set(state);
      autoSyncReady = true;
      if (state.config.mode !== "disabled" && state.config.auto_save && !state.last_synced_at) {
        hasPendingChangesForCloud = true;
        setTimeout(() => void runBackgroundAutoSync(), 1500);
      }
    })
    .catch((error) => backupError.set(describeError(error)));

  for (const unsub of autoSyncUnsubscribes) unsub();
  autoSyncUnsubscribes = [
    bookmarks.subscribe(notifyPersistentDataChanged),
    bookmarkFolders.subscribe(notifyPersistentDataChanged),
    workspaces.subscribe(notifyPersistentDataChanged),
    tabs.subscribe((items) => {
      const sig = items
        .filter((t) => !t.is_ephemeral)
        .map((t) => `${t.id}|${t.url}|${t.workspace_id}|${t.is_pinned ? 1 : 0}|${t.container_id ?? ""}`)
        .join("\n");
      if (sig === lastTabsSignature) return;
      lastTabsSignature = sig;
      const hasDriveTab = items.some(
        (t) =>
          !t.is_ephemeral &&
          t.url.includes("drive.google.com") &&
          !t.url.includes("accounts.google.com")
      );
      if (hasDriveTab) {
        const shouldImmediateSync =
          awaitingGoogleAccountSelection ||
          gdriveLoginRequiredPause ||
          (get(backupSyncState).config.mode === "google_drive_web" &&
            !get(backupSyncState).last_synced_at);
        gdriveLoginRequiredPause = false;
        awaitingGoogleAccountSelection = false;
        if (shouldImmediateSync) {
          setTimeout(() => void runBackgroundAutoSync(true), 1200);
        }
      }
      notifyPersistentDataChanged();
    }),
    splitLayout.subscribe(notifyPersistentDataChanged),
    tabLayout.subscribe(notifyPersistentDataChanged),
    showBookmarksBar.subscribe(notifyPersistentDataChanged),
    overlayBlurPx.subscribe(notifyPersistentDataChanged),
    workspaceSwitcherDisplay.subscribe(notifyPersistentDataChanged),
    defaultSearchEngine.subscribe(notifyPersistentDataChanged),
    autoHibernateMode.subscribe(notifyPersistentDataChanged),
    residentTabLimit.subscribe(notifyPersistentDataChanged),
    hibernationExemptions.subscribe(notifyPersistentDataChanged),
    askDownloadDestination.subscribe(notifyPersistentDataChanged),
    languagePreference.subscribe(notifyPersistentDataChanged),
    keyboardShortcuts.subscribe(notifyPersistentDataChanged),
    pipWindowShadow.subscribe(notifyPersistentDataChanged),
    workspaceIsolation.subscribe(notifyPersistentDataChanged),
  ];

  if (cloudMinuteInterval) clearInterval(cloudMinuteInterval);
  cloudMinuteInterval = setInterval(() => {
    const cfg = get(backupSyncState).config;
    if (
      (cfg.mode === "direct_webdav" || cfg.mode === "google_drive_web") &&
      cfg.auto_save &&
      hasPendingChangesForCloud
    ) {
      void runBackgroundAutoSync();
    }
  }, 60_000);
}

export function dismissBackupPreview(): void {
  const token = source;
  source = null;
  backupPreview.set(null);
  if (token)
    void discardBackupImport(token).catch((error) => backupError.set(describeError(error)));
}
