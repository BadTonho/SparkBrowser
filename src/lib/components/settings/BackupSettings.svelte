<script lang="ts">
  import { t, formatDate } from "../../i18n";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import {
    backupBusy,
    backupError,
    backupExported,
    backupResult,
    backupSyncNotice,
    backupSyncState,
    openGoogleLoginTab,
    pendingBackupPreferences,
    pickBackupSyncFolder,
    retryBackupPreferences,
    saveBackup,
    saveBackupSyncSettings,
    selectBackup,
    syncPullDirect,
    syncPullPreview,
    syncPushNow,
    undoLastReplace,
  } from "../../stores/backupStore";
  import GoogleDriveIcon from "../icons/GoogleDriveIcon.svelte";
  import BackupCard from "./backup/BackupCard.svelte";
  import BackupLocalPanel from "./BackupLocalPanel.svelte";
  import BookmarkExportCard from "./BookmarkExportCard.svelte";
  import BackupPreviewPanel from "./BackupPreviewPanel.svelte";
  import BackupUnlockPrompt from "./BackupUnlockPrompt.svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { errorDetails } from "../../services/user-errors.js";

  const desktop = isTauriRuntime();
  let syncExpanded = $state(false);
  let syncPanel = $state<HTMLDetailsElement | null>(null);

  /** The open panel floats above the page, so Escape or a click outside closes it like a popover. */
  function closeSyncOnEscape(event: KeyboardEvent): void {
    if (event.key !== "Escape" || !syncExpanded) return;
    event.stopPropagation();
    syncExpanded = false;
    syncPanel?.querySelector<HTMLElement>("summary")?.focus();
  }
  const SYNC_OPEN_WIDTH = 580;
  const SYNC_EDGE = 12;
  function scrollParent(element: HTMLElement): HTMLElement {
    for (let node = element.parentElement; node; node = node.parentElement) {
      const overflow = getComputedStyle(node).overflowY;
      if (overflow === "auto" || overflow === "scroll") return node;
    }
    return document.documentElement;
  }
  /**
   * Opens the panel centered on its card and nudged to stay inside the visible settings area, so it
   * grows left, right, up and down as far as there is room. Offsets are relative to the card's slot.
   */
  function placeSyncPanel(): void {
    const slot = syncPanel?.parentElement;
    if (!syncPanel || !slot) return;
    const area = scrollParent(slot).getBoundingClientRect();
    const card = slot.getBoundingClientRect();
    const width = Math.min(SYNC_OPEN_WIDTH, area.width - SYNC_EDGE * 2);
    const left = Math.min(Math.max(card.left + (card.width - width) / 2, area.left + SYNC_EDGE), area.right - SYNC_EDGE - width);
    const top = Math.max(card.top - 28, area.top + SYNC_EDGE);
    const style = syncPanel.style;
    style.setProperty("--sync-left", `${left - card.left}px`);
    style.setProperty("--sync-right", `${card.right - (left + width)}px`);
    style.setProperty("--sync-top", `${top - card.top}px`);
    style.setProperty("--sync-max-height", `${Math.max(240, area.bottom - SYNC_EDGE - top)}px`);
  }
  $effect(() => {
    if (syncExpanded) placeSyncPanel();
  });

  let syncActionBusy = $state(false);
  let syncActionError = $state<string | null>(null);
  const syncProblem = $derived(syncActionError || $backupSyncState.last_error);
  /** The account is not linked yet: a step to take, not a failure, so it is shown as guidance. */
  const linkNeeded = $derived(Boolean(syncProblem) && errorDetails(syncProblem).code === "backup.sync_gdrive_login_required");

  let selectedMode = $state<"disabled" | "google_drive_web" | "folder" | "direct_webdav">("disabled");
  let folderPath = $state("");
  let webdavUrl = $state("");
  let webdavUsername = $state("");
  let webdavSecret = $state("");
  let rememberSecret = $state(true);
  let autoSave = $state(true);
  let directApplyPrefs = $state(false);

  $effect(() => {
    const cfg = $backupSyncState.config;
    selectedMode = cfg.mode;
    if (cfg.mode === "google_drive_web") {
      autoSave = cfg.auto_save;
      directApplyPrefs = cfg.apply_preferences_on_direct_pull;
    } else if (cfg.mode === "folder") {
      folderPath = cfg.directory_path;
      autoSave = cfg.auto_save;
      directApplyPrefs = cfg.apply_preferences_on_direct_pull;
    } else if (cfg.mode === "direct_webdav") {
      webdavUrl = cfg.endpoint_url;
      webdavUsername = cfg.username;
      rememberSecret = cfg.remember_secret;
      autoSave = cfg.auto_save;
      directApplyPrefs = cfg.apply_preferences_on_direct_pull;
    }
  });

  function handleModeChange(event: Event): void {
    const next = (event.currentTarget as HTMLSelectElement).value as
      | "disabled"
      | "google_drive_web"
      | "folder"
      | "direct_webdav";
    selectedMode = next;
    syncActionError = null;
    if (next === "disabled") {
      void saveBackupSyncSettings({ mode: "disabled" }, null);
    } else if (next === "google_drive_web") {
      void saveBackupSyncSettings(
        {
          mode: "google_drive_web",
          auto_save: autoSave,
          apply_preferences_on_direct_pull: directApplyPrefs,
        },
        null
      );
    }
  }

  function handleGDriveOptionsChange(): void {
    void saveBackupSyncSettings(
      {
        mode: "google_drive_web",
        auto_save: autoSave,
        apply_preferences_on_direct_pull: directApplyPrefs,
      },
      null
    );
  }

  function handleFolderOptionsChange(): void {
    if (!folderPath) return;
    void saveBackupSyncSettings(
      {
        mode: "folder",
        directory_path: folderPath,
        auto_save: autoSave,
        apply_preferences_on_direct_pull: directApplyPrefs,
      },
      null
    );
  }

  function handleSaveWebdav(): void {
    const secretToSend = webdavSecret.trim().length > 0 ? webdavSecret : null;
    void saveBackupSyncSettings(
      {
        mode: "direct_webdav",
        endpoint_url: webdavUrl.trim(),
        username: webdavUsername.trim(),
        remember_secret: rememberSecret,
        auto_save: autoSave,
        apply_preferences_on_direct_pull: directApplyPrefs,
      },
      secretToSend
    ).then(() => {
      webdavSecret = "";
    });
  }

  async function runSyncAction(action: () => Promise<void>): Promise<void> {
    syncActionError = null;
    syncActionBusy = true;
    try {
      await action();
      syncActionError = $backupError || null;
      backupError.set("");
    } finally {
      syncActionBusy = false;
    }
  }
</script>

<svelte:window onresize={() => syncExpanded && placeSyncPanel()} />

<section class="setting-group backup-settings" aria-labelledby="backup-heading">
  <h2 class="group-title" id="backup-heading">{$t("backup.heading")}</h2>
  {#if !desktop}<p role="status">{$t("common.desktop_required")}</p>
  {:else}
    <div class="backup-cards">
      <BackupCard id="backup-manual-heading" icon="download" title={$t("backup.manual_heading")} description={$t("backup.manual_description")}>
        {#snippet actions()}
          <button class="pill-action primary-pill" data-testid="backup-export" disabled={$backupBusy} onclick={saveBackup}>{$t("backup.export")}</button>
          <button class="pill-action" data-testid="backup-select" disabled={$backupBusy} onclick={selectBackup}>{$t("backup.import")}</button>
        {/snippet}
      </BackupCard>

      <BookmarkExportCard />

      <!-- The slot keeps the closed card's place in the row; the panel floats over it when open. -->
      <div class="sync-slot">
      <!--
        Click-outside is a real (invisible) button, not a window listener: the settings are shown in a separate
        native surface that only forwards clicks on buttons, so a pointer listener here would never fire.
      -->
      {#if syncExpanded}
        <button type="button" class="sync-scrim" tabindex="-1" aria-label={$t("backup.sync_close")} data-testid="backup-sync-scrim" onclick={() => (syncExpanded = false)}></button>
      {/if}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <details class="sync-panel" bind:this={syncPanel} bind:open={syncExpanded} onkeydown={closeSyncOnEscape} data-testid="backup-sync-panel">
        <summary class="sync-compact-row" data-testid="backup-sync-settings-toggle" title={$t("backup.sync_settings")}>
          <GoogleDriveIcon size={38} />
          <span class="sync-state-chip connection" class:connected={Boolean($backupSyncState.connected_account)} role="status">
            {$t($backupSyncState.connected_account ? "backup.sync_connection_connected" : "backup.sync_connection_disconnected")}
          </span>
          {#if syncActionBusy}
            <span class="sync-state-chip busy" role="status">{$t("backup.sync_status_busy")}</span>
          {:else if linkNeeded}
            <span class="sync-state-chip warn" role="status">{$t("backup.sync_status_link_needed")}</span>
          {:else if syncProblem}
            <span class="sync-state-chip error" role="alert" title={syncProblem || undefined}>{$t("backup.sync_status_error")}</span>
          {:else if $backupSyncState.config.mode === "disabled"}
            <span class="sync-state-chip neutral" role="status">{$t("backup.sync_status_not_configured")}</span>
          {:else if $backupSyncNotice === "pushed"}
            <span class="sync-state-chip synced" role="status" data-testid="backup-sync-notice">{$t("backup.sync_pushed")}</span>
          {:else if $backupSyncNotice === "unchanged"}
            <span class="sync-state-chip neutral" role="status" data-testid="backup-sync-notice">{$t("backup.sync_unchanged")}</span>
          {:else if $backupSyncState.last_synced_at}
            <span class="sync-state-chip synced" role="status" data-testid="backup-sync-last-time">
              {$t("backup.sync_last_synced", {
                time: $formatDate($backupSyncState.last_synced_at * 1000, {
                  dateStyle: "short",
                  timeStyle: "medium",
                }),
              })}
            </span>
          {:else}
            <span class="sync-state-chip neutral" role="status">{$t("backup.sync_status_not_synced")}</span>
          {/if}
        </summary>

        <div class="sync-settings" id="backup-sync-settings">
        <div class="sync-panel-heading">
          <div>
            <h3>{selectedMode === "google_drive_web" ? $t("backup.sync_gdrive_heading") : $t("backup.sync_heading")}</h3>
            <p>{$t("backup.sync_description")}</p>
          </div>
        </div>

        <label class="sync-field sync-mode-field">
          <span>{$t("backup.sync_mode_label")}</span>
          <select
            data-testid="backup-sync-mode"
            value={selectedMode}
            disabled={$backupBusy}
            onchange={handleModeChange}
          >
            <option value="disabled">{$t("backup.sync_mode_disabled")}</option>
            <option value="google_drive_web">{$t("backup.sync_mode_gdrive")}</option>
            <option value="direct_webdav">{$t("backup.sync_mode_webdav")}</option>
            <option value="folder">{$t("backup.sync_mode_folder")}</option>
          </select>
        </label>

      {#if selectedMode === "google_drive_web"}
        <div class="sync-mode-box">
          <div
            class="gdrive-account-card"
            class:connected={Boolean($backupSyncState.connected_account)}
            data-testid="backup-sync-gdrive-account-card"
          >
            <div class="gdrive-avatar" aria-hidden="true">
              {#if $backupSyncState.connected_account}
                {$backupSyncState.connected_account.slice(0, 1).toUpperCase()}
              {:else}
                G
              {/if}
              <span
                class="gdrive-status-dot"
                class:online={Boolean($backupSyncState.connected_account)}
              ></span>
            </div>
            <div class="gdrive-account-info">
              {#if $backupSyncState.connected_account}
                <span class="gdrive-account-label">{$t("backup.sync_gdrive_account_connected")}</span>
                <strong class="gdrive-account-email" data-testid="backup-sync-gdrive-email">
                  {$backupSyncState.connected_account}
                </strong>
                <span class="gdrive-account-sub">{$t("backup.sync_gdrive_target_file")}</span>
              {:else}
                <span class="gdrive-account-label">{$t("backup.sync_gdrive_account_pending")}</span>
                <span class="gdrive-account-sub">{$t("backup.sync_gdrive_account_sub")}</span>
              {/if}
            </div>
            <div class="gdrive-account-actions">
              <button
                class="pill-action primary-pill"
                data-testid="backup-sync-open-google-login"
                disabled={$backupBusy}
                onclick={openGoogleLoginTab}
              >
                {$t("backup.sync_open_google_login")}
              </button>
            </div>
          </div>

          <p class="sync-hint">{$t("backup.sync_gdrive_hint")}</p>

          <div class="sync-options-group">
            <label class="preference-choice">
              <input
                type="checkbox"
                data-testid="backup-sync-auto-save"
                bind:checked={autoSave}
                disabled={$backupBusy}
                onchange={handleGDriveOptionsChange}
              />
              <span>{$t("backup.sync_auto_gdrive")}</span>
            </label>
            <label class="preference-choice">
              <input
                type="checkbox"
                data-testid="backup-sync-direct-prefs"
                bind:checked={directApplyPrefs}
                disabled={$backupBusy}
                onchange={handleGDriveOptionsChange}
              />
              <span>{$t("backup.sync_direct_apply_prefs")}</span>
            </label>
          </div>
        </div>
      {:else if selectedMode === "folder"}
        <div class="sync-mode-box">
          <p class="sync-path" data-testid="backup-sync-folder-path">
            {folderPath ? $t("backup.sync_folder_current", { path: folderPath }) : $t("backup.sync_folder_none")}
          </p>
          <div class="actions">
            <button
              class="pill-action"
              data-testid="backup-sync-choose-folder"
              disabled={$backupBusy}
              onclick={() => pickBackupSyncFolder(autoSave, directApplyPrefs)}
            >
              {$t("backup.sync_choose_folder")}
            </button>
          </div>
          <label class="preference-choice">
            <input
              type="checkbox"
              data-testid="backup-sync-auto-save"
              bind:checked={autoSave}
              disabled={$backupBusy}
              onchange={handleFolderOptionsChange}
            />
            {$t("backup.sync_auto_folder")}
          </label>
          <label class="preference-choice">
            <input
              type="checkbox"
              data-testid="backup-sync-direct-prefs"
              bind:checked={directApplyPrefs}
              disabled={$backupBusy}
              onchange={handleFolderOptionsChange}
            />
            {$t("backup.sync_direct_apply_prefs")}
          </label>
        </div>
      {:else if selectedMode === "direct_webdav"}
        <div class="sync-mode-box">
          <label class="sync-field">
            <span>{$t("backup.sync_webdav_url")}</span>
            <input
              type="url"
              data-testid="backup-sync-webdav-url"
              bind:value={webdavUrl}
              placeholder={$t("backup.sync_webdav_url_placeholder")}
              disabled={$backupBusy}
              autocomplete="off"
              spellcheck="false"
            />
          </label>
          <label class="sync-field">
            <span>{$t("backup.sync_webdav_username")}</span>
            <input
              type="text"
              data-testid="backup-sync-webdav-username"
              bind:value={webdavUsername}
              disabled={$backupBusy}
              autocomplete="off"
              spellcheck="false"
            />
          </label>
          <label class="sync-field">
            <span>{$t("backup.sync_webdav_secret")}</span>
            <input
              type="password"
              data-testid="backup-sync-webdav-secret"
              bind:value={webdavSecret}
              placeholder={$backupSyncState.has_secret ? $t("backup.sync_webdav_secret_saved") : ""}
              disabled={$backupBusy}
              autocomplete="off"
            />
          </label>
          <label class="preference-choice">
            <input
              type="checkbox"
              data-testid="backup-sync-remember-secret"
              bind:checked={rememberSecret}
              disabled={$backupBusy}
            />
            {$t("backup.sync_remember_secret")}
          </label>
          <label class="preference-choice">
            <input
              type="checkbox"
              data-testid="backup-sync-auto-save"
              bind:checked={autoSave}
              disabled={$backupBusy}
            />
            {$t("backup.sync_auto_webdav")}
          </label>
          <label class="preference-choice">
            <input
              type="checkbox"
              data-testid="backup-sync-direct-prefs"
              bind:checked={directApplyPrefs}
              disabled={$backupBusy}
            />
            {$t("backup.sync_direct_apply_prefs")}
          </label>
          <div class="actions">
            <button
              class="pill-action"
              data-testid="backup-sync-save-webdav"
              disabled={$backupBusy}
              onclick={handleSaveWebdav}
            >
              {$t("backup.sync_save_config")}
            </button>
          </div>
        </div>
      {/if}

      {#if $backupSyncState.config.mode !== "disabled"}
        <div class="actions sync-actions-bar">
          <button
            class="pill-action primary-pill"
            data-testid="backup-sync-push"
            disabled={$backupBusy}
            onclick={() => void runSyncAction(syncPushNow)}
          >
            {$t("backup.sync_push_now")}
          </button>
          <button
            class="pill-action"
            data-testid="backup-sync-pull-preview"
            disabled={$backupBusy}
            onclick={() => void runSyncAction(syncPullPreview)}
          >
            {$t("backup.sync_pull_preview")}
          </button>
          <button
            class="pill-action"
            data-testid="backup-sync-pull-direct"
            disabled={$backupBusy}
            onclick={() => void runSyncAction(() => syncPullDirect(directApplyPrefs))}
          >
            {$t("backup.sync_pull_direct")}
          </button>
        </div>
      {/if}

        </div>
        {#if syncProblem}
          <div class="sync-error" class:notice={linkNeeded} data-testid="backup-sync-error">
            <ErrorMessage error={syncProblem} />
          </div>
        {/if}
      </details>
      </div>
    </div>

    <BackupUnlockPrompt />
    <BackupLocalPanel />

    <ErrorMessage error={$backupError} />
    {#if $backupExported}<p role="status">{$t("backup.exported")}</p>{/if}
    {#if $pendingBackupPreferences}
      <aside role="status"><p>{$t("backup.pending")}</p><ErrorMessage error={$pendingBackupPreferences.error} /><button data-testid="backup-retry-preferences" disabled={$backupBusy} onclick={retryBackupPreferences}>{$t("backup.retry")}</button></aside>
    {/if}
    <BackupPreviewPanel />
    {#if $backupResult}
      <p role="status" data-testid="backup-result">{$t("backup.imported")}</p>
      {#if $backupResult.safety_snapshot}
        <p role="status" data-testid="backup-safety-note">{$t("backup.safety_saved")}</p>
        <div class="actions"><button class="pill-action" data-testid="backup-undo-replace" disabled={$backupBusy} onclick={undoLastReplace}>{$t("backup.undo_replace")}</button></div>
      {/if}
    {/if}
  {/if}
</section>
<style>
  p { font-size: 12px; line-height: 1.6; overflow-wrap: anywhere; }
  p { color: var(--text-secondary); }
  h3 { margin: 0; color: var(--text-primary); font-size: 16px; line-height: 1.35; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 10px 0; }
  aside { border: 1px solid var(--border-subtle); border-radius: 12px; padding: 14px; margin-top: 0; background: var(--bg-glass, rgba(255, 255, 255, 0.025)); }
  /* Three cards of the same closed height. Rows do not stretch, so opening the Drive card grows only that card. */
  .backup-cards { --backup-card-min-height: 148px; display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 250px), 280px)); align-items: start; justify-content: start; gap: 12px; }
  /* Closed: exactly the size of its neighbours. Open: grows on every side over the page without moving anything. */
  .sync-scrim { position: fixed; inset: 0; z-index: 29; padding: 0; border: 0; background: transparent; cursor: default; }
  .sync-slot { position: relative; min-width: 0; height: var(--backup-card-min-height, 148px); }
  .sync-panel { position: absolute; top: 0; right: 0; left: 0; display: block; box-sizing: border-box; height: var(--backup-card-min-height, 148px); padding: 16px; overflow: hidden; border: 1px solid var(--border-subtle); border-radius: 14px; background: var(--bg-glass, rgba(255,255,255,.025)); interpolate-size: allow-keywords; transition: top 0.28s ease, right 0.28s ease, left 0.28s ease, height 0.28s ease, background-color 0.28s ease, box-shadow 0.28s ease; }
  .sync-panel[open] { top: var(--sync-top, -28px); right: var(--sync-right, -150px); left: var(--sync-left, -150px); z-index: 30; height: auto; max-height: var(--sync-max-height, 640px); overflow-y: auto; background: var(--bg-permission-popover, #1c1c1c); box-shadow: 0 24px 64px rgba(0, 0, 0, 0.55); }
  /* Keeps the content visible while the panel shrinks back (Chromium/WebView2; elsewhere it just snaps). */
  .sync-panel::details-content { transition: content-visibility 0.28s allow-discrete; }
  @media (prefers-reduced-motion: reduce) { .sync-panel, .sync-panel::details-content { transition: none; } }
  /* Closed: exactly the size of its neighbours. Open: grows on every side over the page without moving anything. */
  .sync-compact-row { display: flex; box-sizing: border-box; min-height: 102px; flex-direction: column; align-items: center; justify-content: center; gap: 6px; padding: 2px; text-align: center; cursor: pointer; list-style: none; }
  .sync-compact-row::-webkit-details-marker, .sync-compact-row::marker { display: none; }
  .sync-compact-row:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 3px; border-radius: 8px; }
  .sync-panel[open] > .sync-compact-row { min-height: 0; flex-direction: row; flex-wrap: wrap; justify-content: flex-start; gap: 8px 10px; padding-bottom: 12px; text-align: left; border-bottom: 1px solid var(--border-subtle); }
  /* Wide panel: description and mode on the left, the mode's fields on the right, sync buttons under the mode. */
  .sync-panel { container-type: inline-size; --text-secondary: color-mix(in srgb, var(--text-primary) 82%, transparent); --sync-error-text: color-mix(in srgb, #ef4444 52%, var(--text-primary)); }
  .sync-panel .pill-action { font-size: 12.5px; }
  .sync-state-chip.warn { border-color: rgba(251, 191, 36, 0.45); background: rgba(251, 191, 36, 0.12); color: #fcd34d; }
  .sync-error.notice { border-color: rgba(251, 191, 36, 0.45); background: rgba(251, 191, 36, 0.08); }
  .sync-error.notice :global(.error-message) { color: color-mix(in srgb, #f59e0b 50%, var(--text-primary)); }
  .sync-error { margin-top: 12px; padding: 12px 14px; border: 1px solid rgba(248, 113, 113, 0.45); border-radius: 10px; background: rgba(248, 113, 113, 0.1); }
  .sync-error :global(.error-message) { color: var(--sync-error-text); font-size: 13px; line-height: 1.6; }
  .sync-error :global(.error-message p) { margin: 0; }
  .sync-error :global(.error-message summary) { margin-top: 8px; width: fit-content; cursor: pointer; font-size: 12.5px; font-weight: 600; text-decoration: underline; text-underline-offset: 3px; }
  .sync-error :global(.error-message pre) { margin: 8px 0 0; padding: 10px 12px; border-radius: 8px; background: rgba(0, 0, 0, 0.3); color: var(--text-primary); font-size: 12px; line-height: 1.5; }
  @container (min-width: 520px) {
    .sync-settings { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.15fr); grid-template-areas: "head box" "mode box" "acts box"; grid-template-rows: auto auto 1fr; align-items: start; gap: 12px 20px; }
    .sync-panel-heading { grid-area: head; }
    .sync-mode-field { grid-area: mode; }
    .sync-mode-box { grid-area: box; }
    .sync-actions-bar { grid-area: acts; align-self: end; }
  }
  .sync-settings { display: flex; width: 100%; min-width: 0; flex-direction: column; gap: 9px; padding-top: 12px; }
  .sync-state-chip { display: inline-flex; max-width: 100%; align-items: center; justify-content: center; padding: 5px 11px; border: 1px solid var(--border-subtle); border-radius: 999px; background: rgba(255,255,255,.04); color: var(--text-secondary); font-size: 12px; line-height: 1.35; overflow-wrap: anywhere; }
  .sync-state-chip.connection.connected, .sync-state-chip.synced { border-color: rgba(34, 197, 94, .35); background: rgba(34, 197, 94, .1); color: #86efac; }
  .sync-state-chip.connection:not(.connected) { border-color: rgba(255,255,255,.12); }
  .sync-state-chip.busy { border-color: rgba(66, 133, 244, .4); background: rgba(66, 133, 244, .12); color: #93c5fd; }
  .sync-state-chip.error { border-color: rgba(248, 113, 113, .42); background: rgba(248, 113, 113, .12); color: #fca5a5; }
  .sync-state-chip.neutral { color: var(--text-secondary); }
  .sync-panel-heading p { margin: 4px 0 0; font-size: 12.5px; line-height: 1.55; }
  .preference-choice { display: flex; align-items: center; gap: 8px; margin: 10px 0; font-size: 12.5px; color: var(--text-primary); cursor: pointer; }
  .sync-field { display: flex; flex-direction: column; gap: 6px; margin: 10px 0; font-size: 12.5px; font-weight: 600; color: var(--text-primary); }
  .sync-field input, .sync-field select { min-width: 0; padding: 7px 9px; border-radius: 7px; border: 1px solid var(--border-subtle); background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 12.5px; }
  .sync-path { font-family: monospace; color: var(--text-primary); padding: 8px 10px; border-radius: 8px; background: rgba(255, 255, 255, 0.03); border: 1px solid var(--border-subtle); }
  .sync-mode-box { display: flex; flex-direction: column; gap: 8px; margin-top: 0; }
  .gdrive-account-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 10px;
    padding: 10px;
    border-radius: 9px;
    border: 1px solid rgba(255, 255, 255, 0.09);
    background: linear-gradient(135deg, rgba(66, 133, 244, 0.08), rgba(52, 168, 83, 0.05));
  }
  .gdrive-account-card.connected {
    border-color: rgba(52, 168, 83, 0.35);
    background: linear-gradient(135deg, rgba(52, 168, 83, 0.12), rgba(66, 133, 244, 0.06));
  }
  .gdrive-avatar {
    position: relative;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: rgba(66, 133, 244, 0.22);
    border: 1px solid rgba(66, 133, 244, 0.45);
    color: var(--text-primary);
    font-weight: 700;
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .gdrive-status-dot {
    position: absolute;
    right: -1px;
    bottom: -1px;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: #f59e0b;
    border: 2px solid #141418;
  }
  .gdrive-status-dot.online {
    background: #22c55e;
  }
  .gdrive-account-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .gdrive-account-label {
    font-size: 11.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-secondary);
  }
  .gdrive-account-email {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .gdrive-account-sub {
    font-size: 12.5px;
    line-height: 1.5;
    color: var(--text-secondary);
  }
  .gdrive-account-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .sync-hint {
    margin: 2px 0;
    font-size: 12.5px;
    line-height: 1.6;
    color: var(--text-secondary);
  }
  .sync-options-group {
    padding: 6px 9px;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.025);
    border: 1px solid var(--border-subtle);
  }
  .primary-pill {
    background: rgba(66, 133, 244, 0.18);
    border-color: rgba(66, 133, 244, 0.42);
    color: var(--text-primary);
    font-weight: 600;
  }
  .primary-pill:hover:not(:disabled) {
    background: rgba(66, 133, 244, 0.28);
  }
  .sync-actions-bar {
    padding-top: 8px;
    margin: 0;
    border-top: 1px solid var(--border-subtle);
  }
  @media (max-width: 520px) {
    aside { padding: 14px; border-radius: 12px; }
    .gdrive-account-card { align-items: flex-start; }
    .gdrive-account-actions { width: 100%; }
    .gdrive-account-actions :global(button) { max-width: 100%; white-space: normal; text-align: left; }
    .sync-actions-bar > button { flex: 1 1 100%; }
  }
</style>
