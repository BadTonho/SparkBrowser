import { createSharedLifecycle } from "../services/lifecycle.js";
import { settleWithin } from "../services/deadline.js";
import { applyWorkspaceSnapshot } from "./workspaceStore";
import type { TabItem } from "../types/browser";
import { applyBackupZoom } from "../services/bridge/zoom";
import { get, writable } from "svelte/store";
import { isTauriRuntime } from "../services/bridge/runtime";
import {
  cancelBackupUnlock,
  chooseBackupFile,
  chooseBackupSyncFolder,
  commitBackupImport,
  createLocalBackup,
  deleteLocalBackup,
  discardBackupImport,
  exportBackup,
  exportBookmarks,
  fetchBackupSyncState,
  fetchLocalBackupState,
  fetchPendingBackupPreferences,
  finishBackupPreferences,
  previewBackupImport,
  pullBackupSync,
  pushBackupSync,
  restoreLocalBackup,
  saveBackupEncryption,
  saveBackupSyncConfig,
  saveLocalBackupSchedule,
  unlockBackupFile,
} from "../services/bridge/backup";
import { applyBackupPreferences, capturePreferences } from "../services/backup/preferences";
import { UserError, describeError } from "../services/user-errors.js";
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
  BookmarkExportFormat,
  BookmarkExportResult,
  LocalBackupReason,
  LocalBackupSchedule,
  LocalBackupState,
  RestoreGroup,
  RestoreOptions,
  SyncStateSnapshot,
  SyncTargetConfig,
} from "../types/backup";

export const backupBusy = writable(false);
export const backupError = writable("");
export const backupPreview = writable<BackupPreview | null>(null);
export const backupResult = writable<BackupImportResult | null>(null);
export const DEFAULT_RESTORE_GROUPS: RestoreGroup[] = ["workspaces", "bookmarks", "sessions", "notes", "site_search"];
export const restoreOptions = writable<RestoreOptions>({ groups: [...DEFAULT_RESTORE_GROUPS], mode: "merge" });
export const backupExported = writable(false);
export const bookmarkExportResult = writable<BookmarkExportResult | null>(null);
export const pendingBackupPreferences = writable<BackupPreferenceTask | null>(null);
export const backupSyncState = writable<SyncStateSnapshot>({
  config: { mode: "disabled" },
  has_secret: false,
  connected_account: null,
  last_synced_at: null,
  last_error: null,
});
export const backupSyncNotice = writable<"pushed" | "unchanged" | "config_saved" | "">("");
/** Set while an encrypted file waits for its password; `wrong` after a failed attempt. */
export const backupLocked = writable<{ wrong: boolean } | null>(null);
export const localBackupState = writable<LocalBackupState>({
  schedule: { enabled: true, interval: "daily", on_exit: true, max_backups: 10, max_age_days: null },
  encryption_enabled: false,
  last_run_at: null,
  last_failure: null,
  next_due_at: null,
  directory: "",
  entries: [],
  total_bytes: 0,
});
export const localBackupBusy = writable(false);
export const localBackupError = writable("");
export const localBackupNotice = writable<"created" | "encryption_saved" | "">("");

let source: string | null = null;
let folderAutoSaveTimer: ReturnType<typeof setTimeout> | undefined;
let cloudMinuteInterval: ReturnType<typeof setInterval> | undefined;
let autoSyncUnsubscribes: (() => void)[] = [];
let hasPendingChangesForCloud = false;
let autoSyncReady = false;
let gdriveLoginRequiredPause = false;
let awaitingGoogleAccountSelection = false;
let googleLoginTabId: string | null = null;
let lastTabsSignature = "";
let localScheduleInterval: ReturnType<typeof setInterval> | undefined;

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

export const exportBookmarksAs = (format: BookmarkExportFormat) =>
  action(async () => {
    bookmarkExportResult.set(null);
    // A cancelled dialog returns null and leaves no notice.
    bookmarkExportResult.set(await exportBookmarks(format));
  });

export const selectBackup = () =>
  action(async () => {
    if (source) await discardBackupImport(source);
    backupResult.set(null);
    backupPreview.set(null);
    source = null;
    backupLocked.set(null);
    resetRestoreOptions();
    let document: string | null;
    try {
      document = await chooseBackupFile();
    } catch (error) {
      if (holdForPassword(error)) return;
      throw error;
    }
    if (!document) return;
    source = document;
    await createPreview();
  });

function errorCode(error: unknown): string {
  return error instanceof UserError ? error.code : "";
}

/** Encrypted backups pause the import and ask for a password instead of failing. */
function holdForPassword(error: unknown): boolean {
  const code = errorCode(error);
  if (code !== "backup.password_required" && code !== "backup.password_wrong") return false;
  backupLocked.set({ wrong: code === "backup.password_wrong" });
  return true;
}

export const unlockBackup = (password: string) =>
  action(async () => {
    try {
      source = await unlockBackupFile(password);
    } catch (error) {
      if (errorCode(error) === "backup.password_wrong") {
        backupLocked.set({ wrong: true });
        return;
      }
      throw error;
    }
    backupLocked.set(null);
    await createPreview();
  });

export function cancelBackupPasswordPrompt(): void {
  backupLocked.set(null);
  void cancelBackupUnlock().catch((error) => backupError.set(describeError(error)));
}

async function createPreview(): Promise<void> {
  if (!source) return;
  await noteWriter.flushAll();
  backupPreview.set(await previewBackupImport(source, capturePreferences(), get(restoreOptions)));
}

export const refreshBackupPreview = () => action(createPreview);

/** Every new file starts from the safe default: all data merged, preferences left alone. */
function resetRestoreOptions(): void {
  restoreOptions.set({ groups: [...DEFAULT_RESTORE_GROUPS], mode: "merge" });
}

/** Changing the selection re-plans the import, so counts always match what will be written. */
export const updateRestoreOptions = (next: RestoreOptions) =>
  action(async () => {
    restoreOptions.set(next);
    // Nothing selected has no plan; confirming stays disabled until a group is picked again.
    if (next.groups.length > 0) await createPreview();
  });

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
  applyWorkspaceSnapshot(result.workspaces);
  applyTabsSnapshot(result.snapshot);
  pendingBackupPreferences.set(result.pending_preferences);
  // The pre-replace safety copy now exists in the history list.
  if (result.safety_snapshot)
    void fetchLocalBackupState()
      .then((state) => localBackupState.set(state))
      .catch((error) => localBackupError.set(describeError(error)));
  await loadBookmarks();
  if (result.pending_preferences) await applyPending();
}

export const importBackup = () =>
  action(async () => {
    const preview = get(backupPreview);
    if (!preview) return;
    // Preferences are queued only when the previewed selection includes settings or shortcuts.
    const applyPreferences = preview.groups.some((group) => group === "settings" || group === "shortcuts");
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
        scheduleAutoSync();
      } else if (config.mode === "google_drive_web" || config.mode === "direct_webdav") {
        gdriveLoginRequiredPause = false;
        setTimeout(() => void runBackgroundAutoSync(), 150);
      }
    }
  });

function finishGoogleAccountSelection(items: TabItem[]): void {
  if (!awaitingGoogleAccountSelection) return;
  const selected = items.find(t => {
    if (t.id !== googleLoginTabId || t.is_ephemeral) return false;
    try { const url = new URL(t.url); return url.origin === "https://drive.google.com" && url.pathname.startsWith("/drive/"); }
    catch { return false; }
  });
  if (awaitingGoogleAccountSelection && selected) {
    awaitingGoogleAccountSelection = false;
    googleLoginTabId = null;
    const config = get(backupSyncState).config;
    if (config.mode === "google_drive_web") {
      void saveBackupSyncConfig(config, null, selected.id).then(state => {
        backupSyncState.set(state);
        gdriveLoginRequiredPause = false;
        hasPendingChangesForCloud = true;
        setTimeout(() => void runBackgroundAutoSync(true), 1200);
      }).catch(error => backupError.set(describeError(error)));
    }
  }
}

export async function openGoogleLoginTab(): Promise<void> {
  gdriveLoginRequiredPause = false;
  awaitingGoogleAccountSelection = true;
  googleLoginTabId = null;
  await closeSettingsModal();
  await addTab(
    "https://accounts.google.com/AccountChooser?continue=https%3A%2F%2Fdrive.google.com%2Fdrive%2Fmy-drive&service=wise"
  );
  googleLoginTabId = get(tabs).find(tab => tab.is_active && !tab.is_ephemeral)?.id ?? null;
  finishGoogleAccountSelection(get(tabs));
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
    resetRestoreOptions();
    await noteWriter.flushAll();
    const res = await pullBackupSync(capturePreferences(), false, null);
    backupSyncState.set(res.state);
    if (res.kind === "preview") {
      source = res.source_token;
      // The pull previews with backend defaults; re-plan with the UI selection (preferences off by default).
      await createPreview();
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

async function localAction(work: () => Promise<void>): Promise<void> {
  if (get(localBackupBusy) || !isTauriRuntime()) return;
  localBackupBusy.set(true);
  localBackupError.set("");
  localBackupNotice.set("");
  try {
    await work();
  } catch (error) {
    localBackupError.set(describeError(error));
  } finally {
    localBackupBusy.set(false);
  }
}

export const refreshLocalBackups = () =>
  localAction(async () => {
    localBackupState.set(await fetchLocalBackupState());
  });

export const saveLocalSchedule = (schedule: LocalBackupSchedule) =>
  localAction(async () => {
    localBackupState.set(await saveLocalBackupSchedule(schedule));
  });

export const saveEncryption = (enabled: boolean, password: string | null = null) =>
  localAction(async () => {
    localBackupState.set(await saveBackupEncryption(enabled, password));
    localBackupNotice.set("encryption_saved");
  });

async function createLocal(reason: LocalBackupReason, onlyIfChanged: boolean): Promise<boolean> {
  await noteWriter.flushAll();
  try {
    const outcome = await createLocalBackup(capturePreferences(), reason, onlyIfChanged);
    localBackupState.set(outcome.state);
    return outcome.created;
  } catch (error) {
    // The backend recorded the failure; reload so the status panel shows it right away.
    await fetchLocalBackupState().then((state) => localBackupState.set(state)).catch(() => {});
    throw error;
  }
}

export const backupNow = () =>
  localAction(async () => {
    if (await createLocal("manual", false)) localBackupNotice.set("created");
  });

export const removeLocalBackup = (id: string) =>
  localAction(async () => {
    localBackupState.set(await deleteLocalBackup(id));
  });

export const restoreFromHistory = (id: string) =>
  action(async () => {
    if (source) await discardBackupImport(source);
    backupResult.set(null);
    backupPreview.set(null);
    source = null;
    backupLocked.set(null);
    resetRestoreOptions();
    try {
      source = await restoreLocalBackup(id);
    } catch (error) {
      if (holdForPassword(error)) return;
      throw error;
    }
    await createPreview();
  });

/** Reopens the pre-replace safety copy as a replace-mode review of the same groups. */
export const undoLastReplace = () =>
  action(async () => {
    const result = get(backupResult);
    const id = result?.safety_snapshot;
    if (!result || !id) return;
    if (source) await discardBackupImport(source);
    backupResult.set(null);
    backupPreview.set(null);
    source = null;
    backupLocked.set(null);
    restoreOptions.set({ groups: [...result.replaced], mode: "replace" });
    try {
      source = await restoreLocalBackup(id);
    } catch (error) {
      if (holdForPassword(error)) return;
      throw error;
    }
    await createPreview();
  });

/** Runs when the schedule is due; failures surface in the panel but never interrupt browsing. */
async function runScheduledLocalBackup(): Promise<void> {
  if (!isTauriRuntime() || get(localBackupBusy) || get(backupBusy)) return;
  const { schedule, next_due_at } = get(localBackupState);
  if (!schedule.enabled || next_due_at === null || Date.now() / 1000 < next_due_at) return;
  localBackupBusy.set(true);
  try {
    await createLocal("scheduled", true);
    localBackupError.set("");
  } catch (error) {
    localBackupError.set(describeError(error));
    // Retry in 15 minutes rather than every minute while the problem persists.
    localBackupState.update((state) => ({ ...state, next_due_at: Math.floor(Date.now() / 1000) + 15 * 60 }));
  } finally {
    localBackupBusy.set(false);
  }
}

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

/**
 * A folder is written almost instantly, so it follows each change closely. Google Drive and WebDAV go over
 * the network (Drive through a hidden page), so they wait for a quiet moment instead of uploading on every
 * edit. Either way the changes are already saved by the time the browser closes, which keeps closing fast:
 * only edits made in the last few seconds are left for the exit sync to finish.
 */
const FOLDER_AUTO_SYNC_DELAY_MS = 1_000;
const REMOTE_AUTO_SYNC_DELAY_MS = 15_000;

function scheduleAutoSync(): void {
  if (folderAutoSaveTimer) clearTimeout(folderAutoSaveTimer);
  const delay = get(backupSyncState).config.mode === "folder" ? FOLDER_AUTO_SYNC_DELAY_MS : REMOTE_AUTO_SYNC_DELAY_MS;
  folderAutoSaveTimer = setTimeout(() => {
    folderAutoSaveTimer = undefined;
    const cfg = get(backupSyncState).config;
    if (cfg.mode !== "disabled" && cfg.auto_save) {
      void runBackgroundAutoSync();
    }
  }, delay);
}

function notifyPersistentDataChanged(): void {
  if (!autoSyncReady) return;
  hasPendingChangesForCloud = true;
  const cfg = get(backupSyncState).config;
  if (cfg.mode !== "disabled" && cfg.auto_save) {
    scheduleAutoSync();
  }
}

/** How long closing the browser waits for the exit backup and the final sync before it gives up on them. */
export const EXIT_BACKUP_WAIT_MS = 8_000;
export const EXIT_SYNC_WAIT_MS = 5_000;

export async function flushBackupSyncBeforeShutdown(): Promise<void> {
  if (!isTauriRuntime()) return;
  // Both steps are best effort and bounded: a slow disk, an unreachable WebDAV server or a Google
  // Drive page that never finishes loading must never keep the browser from closing.
  if (get(localBackupState).schedule.on_exit) {
    await settleWithin(createLocal("exit", true), EXIT_BACKUP_WAIT_MS);
  }
  if (folderAutoSaveTimer) {
    clearTimeout(folderAutoSaveTimer);
    folderAutoSaveTimer = undefined;
  }
  const cfg = get(backupSyncState).config;
  if (cfg.mode === "disabled" || !cfg.auto_save) return;
  const pushed = await settleWithin(pushBackupSync(capturePreferences(), true), EXIT_SYNC_WAIT_MS);
  if ("value" in pushed) {
    backupSyncState.set(pushed.value.state);
    hasPendingChangesForCloud = false;
  }
}

function startBackupStatus(scope: import("../services/lifecycle.js").LifecycleScope): () => void {
  if (!isTauriRuntime()) return () => {};
  let startupTimer: ReturnType<typeof setTimeout> | undefined;
  let startupLocalTimer: ReturnType<typeof setTimeout> | undefined;
  void fetchPendingBackupPreferences()
    .then(scope.guard((value) => pendingBackupPreferences.set(value)))
    .catch(scope.guard((error) => backupError.set(describeError(error))));

  void fetchBackupSyncState()
    .then(scope.guard((state) => {
      backupSyncState.set(state);
      autoSyncReady = true;
      if (state.config.mode !== "disabled" && state.config.auto_save && !state.last_synced_at) {
        hasPendingChangesForCloud = true;
        startupTimer = setTimeout(() => { if (scope.active()) void runBackgroundAutoSync(); }, 1500);
      }
    }))
    .catch(scope.guard((error) => backupError.set(describeError(error))));

  void fetchLocalBackupState()
    .then(scope.guard((state) => {
      localBackupState.set(state);
      // Catch up after the browser was closed while a backup came due.
      startupLocalTimer = setTimeout(() => { if (scope.active()) void runScheduledLocalBackup(); }, 20_000);
    }))
    .catch(scope.guard((error) => localBackupError.set(describeError(error))));
  if (localScheduleInterval) clearInterval(localScheduleInterval);
  localScheduleInterval = setInterval(() => void runScheduledLocalBackup(), 60_000);

  for (const unsub of autoSyncUnsubscribes) unsub();
  autoSyncUnsubscribes = [
    bookmarks.subscribe(notifyPersistentDataChanged),
    bookmarkFolders.subscribe(notifyPersistentDataChanged),
    workspaces.subscribe(notifyPersistentDataChanged),
    tabs.subscribe((items) => {
      finishGoogleAccountSelection(items);
      const sig = items
        .filter((t) => !t.is_ephemeral)
        .map((t) => `${t.id}|${t.url}|${t.workspace_id}|${t.is_pinned ? 1 : 0}|${t.container_id ?? ""}`)
        .join("\n");
      if (sig === lastTabsSignature) return;
      lastTabsSignature = sig;
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
  return () => {
    autoSyncReady = false;
    for (const stop of autoSyncUnsubscribes.splice(0)) stop();
    clearInterval(cloudMinuteInterval);
    clearTimeout(folderAutoSaveTimer);
    clearTimeout(startupTimer);
    clearTimeout(startupLocalTimer);
    clearInterval(localScheduleInterval);
    localScheduleInterval = undefined;
    cloudMinuteInterval = undefined;
    folderAutoSaveTimer = undefined;
  };
}

export function dismissBackupPreview(): void {
  const token = source;
  source = null;
  backupPreview.set(null);
  if (token)
    void discardBackupImport(token).catch((error) => backupError.set(describeError(error)));
}

export const initializeBackupStatus = createSharedLifecycle(scope => scope.own(startBackupStatus(scope)));
