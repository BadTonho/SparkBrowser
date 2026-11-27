<script lang="ts">
  import { t, formatDate, type MessageKey } from "../../i18n";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import {
    backupBusy,
    backupError,
    backupExported,
    backupResult,
    backupSyncFinished,
    backupSyncNotice,
    backupSyncRunning,
    driveWaitingForAccount,
    saveSyncOptions,
    syncPausedNow,
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
  import AppIcon from "../icons/AppIcon.svelte";
  import GoogleDriveIcon from "../icons/GoogleDriveIcon.svelte";
  import BackupCard from "./backup/BackupCard.svelte";
  import BackupLocalPanel from "./BackupLocalPanel.svelte";
  import BookmarkExportCard from "./BookmarkExportCard.svelte";
  import BackupPreviewPanel from "./BackupPreviewPanel.svelte";
  import BackupUnlockPrompt from "./BackupUnlockPrompt.svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { errorDetails } from "../../services/user-errors.js";
  import { gdriveFix } from "../../services/backup/gdrive-fix.js";
  import { summarizeSyncRing } from "../../services/backup/sync-ring";
  import { pauseUntil } from "../../services/backup/sync-schedule";
  import type { RestoreGroup, SyncInterval } from "../../types/backup";

  const INTERVALS: SyncInterval[] = ["auto", "five_minutes", "fifteen_minutes", "on_exit"];
  const SYNC_GROUPS: RestoreGroup[] = ["workspaces", "bookmarks", "sessions", "notes", "site_search", "settings", "shortcuts"];

  /** Turns a group on or off; the last group left stays on (there must be something to sync). */
  function toggleSyncGroup(group: RestoreGroup, on: boolean): void {
    const current = $backupSyncState.options.groups;
    const next = on ? SYNC_GROUPS.filter((item) => item === group || current.includes(item)) : current.filter((item) => item !== group);
    if (next.length) void saveSyncOptions({ groups: next });
  }

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
  // Where the open panel goes is decided by CSS alone: the settings are usually projected onto a
  // separate surface that copies classes but not inline styles, so positions set from script never
  // reach the screen.

  let syncActionBusy = $state(false);
  let syncActionError = $state<string | null>(null);
  const syncProblem = $derived(syncActionError || $backupSyncState.last_error);
  const driveLogin = $derived(Boolean(syncProblem) && errorDetails(syncProblem).code === "backup.sync_gdrive_login_required");
  /** What to do about a Drive login error, chosen from the reason the native side attached. */
  const driveFix = $derived(driveLogin ? gdriveFix(errorDetails(syncProblem).detail) : null);
  /** Only a missing link is guidance rather than a failure; every other reason is shown as an error. */
  const linkNeeded = $derived(driveFix === "link");
  /** A sync runs now: a button of the panel, or the automatic one in the background. */
  const syncing = $derived(syncActionBusy || $backupSyncRunning > 0);
  /** The ring around the closed card, like the download button's. Classes only: projection drops inline styles. */
  const ring = $derived.by(() => {
    const config = $backupSyncState.config;
    return summarizeSyncRing({
      mode: config.mode,
      autoSave: "auto_save" in config ? config.auto_save : false,
      running: syncing ? 1 : 0,
      finished: $backupSyncFinished,
      problem: syncProblem || null,
      waitingForAccount: linkNeeded || $driveWaitingForAccount,
      pausedByUser: $syncPausedNow,
    });
  });

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

<!-- An option of the sync panel: what it does on the left, the same switch as the rest of Settings on the right. -->
{#snippet switchRow(label: string, description: string, testid: string, checked: boolean, onchange: (checked: boolean) => void)}
  <label class="sync-row">
    <span class="sync-row-text"><span class="sync-row-label">{label}</span><span class="sync-row-desc">{description}</span></span>
    <input type="checkbox" role="switch" data-testid={testid} {checked} disabled={$backupBusy} onchange={(event) => onchange(event.currentTarget.checked)} />
  </label>
{/snippet}

<!-- Saved on its own (sync options), so it applies at once in every mode, WebDAV included. -->
{#snippet pullOnStartRow()}
  {@render switchRow($t("backup.sync_pull_on_start_label"), $t("backup.sync_pull_on_start_desc"), "backup-sync-pull-on-start", $backupSyncState.options.pull_on_start, (on) => void saveSyncOptions({ pull_on_start: on }))}
{/snippet}

<section class="setting-group backup-settings" aria-labelledby="backup-heading">
  <h2 class="group-title" id="backup-heading">{$t("backup.heading")}</h2>
  {#if !desktop}<p role="status">{$t("common.desktop_required")}</p>
  {:else}
    <div class="setting-subgroup sync-group" role="group" aria-labelledby="backup-sync-group-heading">
      <h3 class="subgroup-title" id="backup-sync-group-heading">{$t("backup.group_sync")}</h3>
      <!-- The slot keeps the closed card's place; the panel floats over it when open. -->
      <div class="sync-slot" class:ring-syncing={!syncExpanded && ring === "syncing"} class:ring-done={!syncExpanded && ring === "done"}
        class:ring-paused={!syncExpanded && ring === "paused"} class:ring-error={!syncExpanded && ring === "error"} data-ring={ring} data-testid="backup-sync-ring">
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
          <span class="sync-icon" aria-hidden="true">
            {#if $backupSyncState.config.mode === "google_drive_web"}<GoogleDriveIcon size={30} />{:else}<AppIcon name="refresh" size={24} />{/if}
          </span>
          <strong class="sync-title" data-testid="backup-sync-title">{$t(`backup.sync_title_${$backupSyncState.config.mode}` as MessageKey)}</strong>
          <span class="sync-chips">
          {#if $backupSyncState.config.mode === "google_drive_web"}
            <span class="sync-state-chip connection" class:connected={Boolean($backupSyncState.connected_account)} role="status">
              {$t($backupSyncState.connected_account ? "backup.sync_connection_connected" : "backup.sync_connection_disconnected")}
            </span>
          {/if}
          {#if syncing}
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
          </span>
        </summary>

        <div class="sync-settings" id="backup-sync-settings">
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
          <small>{$t("backup.sync_description")}</small>
        </label>

      {#if selectedMode === "google_drive_web"}
        <section class="sync-section" aria-labelledby="sync-account-title">
          <h4 class="sync-section-title" id="sync-account-title">{$t("backup.sync_section_account")}</h4>
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
        </section>

        <section class="sync-section" aria-labelledby="sync-auto-title">
          <h4 class="sync-section-title" id="sync-auto-title">{$t("backup.sync_section_auto")}</h4>
          <div class="sync-rows">
            {@render switchRow($t("backup.sync_auto_label"), $t("backup.sync_auto_gdrive"), "backup-sync-auto-save", autoSave, (on) => { autoSave = on; handleGDriveOptionsChange(); })}
            {@render pullOnStartRow()}
            {@render switchRow($t("backup.sync_direct_prefs_label"), $t("backup.sync_direct_apply_prefs"), "backup-sync-direct-prefs", directApplyPrefs, (on) => { directApplyPrefs = on; handleGDriveOptionsChange(); })}
          </div>
        </section>
      {:else if selectedMode === "folder"}
        <section class="sync-section" aria-labelledby="sync-folder-title">
          <h4 class="sync-section-title" id="sync-folder-title">{$t("backup.sync_section_folder")}</h4>
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
        </section>

        <section class="sync-section" aria-labelledby="sync-auto-title">
          <h4 class="sync-section-title" id="sync-auto-title">{$t("backup.sync_section_auto")}</h4>
          <div class="sync-rows">
            {@render switchRow($t("backup.sync_auto_label"), $t("backup.sync_auto_folder"), "backup-sync-auto-save", autoSave, (on) => { autoSave = on; handleFolderOptionsChange(); })}
            {@render pullOnStartRow()}
            {@render switchRow($t("backup.sync_direct_prefs_label"), $t("backup.sync_direct_apply_prefs"), "backup-sync-direct-prefs", directApplyPrefs, (on) => { directApplyPrefs = on; handleFolderOptionsChange(); })}
          </div>
        </section>
      {:else if selectedMode === "direct_webdav"}
        <section class="sync-section" aria-labelledby="sync-connection-title">
          <h4 class="sync-section-title" id="sync-connection-title">{$t("backup.sync_section_connection")}</h4>
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
          <div class="sync-rows">
            {@render switchRow($t("backup.sync_remember_secret"), $t("backup.sync_remember_secret_desc"), "backup-sync-remember-secret", rememberSecret, (on) => (rememberSecret = on))}
          </div>
        </section>

        <section class="sync-section" aria-labelledby="sync-auto-title">
          <h4 class="sync-section-title" id="sync-auto-title">{$t("backup.sync_section_auto")}</h4>
          <div class="sync-rows">
            {@render switchRow($t("backup.sync_auto_label"), $t("backup.sync_auto_webdav"), "backup-sync-auto-save", autoSave, (on) => (autoSave = on))}
            {@render pullOnStartRow()}
            {@render switchRow($t("backup.sync_direct_prefs_label"), $t("backup.sync_direct_apply_prefs"), "backup-sync-direct-prefs", directApplyPrefs, (on) => (directApplyPrefs = on))}
          </div>
          <div class="actions">
            <button
              class="pill-action primary-pill"
              data-testid="backup-sync-save-webdav"
              disabled={$backupBusy}
              onclick={handleSaveWebdav}
            >
              {$t("backup.sync_save_config")}
            </button>
          </div>
        </section>
      {/if}

      {#if $backupSyncState.config.mode !== "disabled"}
        <section class="sync-section" aria-labelledby="sync-groups-title">
          <h4 class="sync-section-title" id="sync-groups-title">{$t("backup.sync_section_groups")}</h4>
          <p class="sync-hint">{$t("backup.sync_groups_hint")}</p>
          <div class="sync-rows" data-testid="backup-sync-groups">
            {#each SYNC_GROUPS as group}
              {@const on = $backupSyncState.options.groups.includes(group)}
              <label class="sync-row">
                <span class="sync-row-text"><span class="sync-row-label">{$t(`backup.group_${group}` as MessageKey)}</span></span>
                <input type="checkbox" role="switch" data-testid={"backup-sync-group-" + group} checked={on}
                  disabled={$backupBusy || (on && $backupSyncState.options.groups.length === 1)} onchange={(event) => toggleSyncGroup(group, event.currentTarget.checked)} />
              </label>
            {/each}
          </div>
        </section>

        <section class="sync-section" aria-labelledby="sync-schedule-title">
          <h4 class="sync-section-title" id="sync-schedule-title">{$t("backup.sync_section_schedule")}</h4>
          <div class="sync-rows">
            <label class="sync-row">
              <span class="sync-row-text"><span class="sync-row-label">{$t("backup.sync_interval_label")}</span><span class="sync-row-desc">{$t("backup.sync_interval_desc")}</span></span>
              <select class="sync-row-select" data-testid="backup-sync-interval" value={$backupSyncState.options.interval} disabled={$backupBusy}
                onchange={(event) => void saveSyncOptions({ interval: event.currentTarget.value as SyncInterval })}>
                {#each INTERVALS as interval}
                  <option value={interval}>{$t(interval === "auto" ? ($backupSyncState.config.mode === "folder" ? "backup.sync_interval_auto_folder" : "backup.sync_interval_auto_remote") : (`backup.sync_interval_${interval}` as MessageKey))}</option>
                {/each}
              </select>
            </label>
            <div class="sync-row" data-testid="backup-sync-pause">
              {#if $syncPausedNow && $backupSyncState.options.paused_until}
                <span class="sync-row-text">
                  <span class="sync-row-label">{$t("backup.sync_paused_until", { time: $formatDate($backupSyncState.options.paused_until * 1000, { dateStyle: "short", timeStyle: "short" }) })}</span>
                  <span class="sync-row-desc">{$t("backup.sync_paused_desc")}</span>
                </span>
                <button class="pill-action primary-pill" data-testid="backup-sync-resume" disabled={$backupBusy} onclick={() => void saveSyncOptions({ paused_until: null })}>{$t("backup.sync_resume")}</button>
              {:else}
                <span class="sync-row-text"><span class="sync-row-label">{$t("backup.sync_pause_label")}</span><span class="sync-row-desc">{$t("backup.sync_pause_desc")}</span></span>
                <span class="sync-row-buttons">
                  <button class="pill-action" data-testid="backup-sync-pause-hour" disabled={$backupBusy} onclick={() => void saveSyncOptions({ paused_until: pauseUntil("hour", new Date()) })}>{$t("backup.sync_pause_hour")}</button>
                  <button class="pill-action" data-testid="backup-sync-pause-tomorrow" disabled={$backupBusy} onclick={() => void saveSyncOptions({ paused_until: pauseUntil("tomorrow", new Date()) })}>{$t("backup.sync_pause_tomorrow")}</button>
                </span>
              {/if}
            </div>
          </div>
        </section>

        <section class="sync-section" aria-labelledby="sync-now-title">
          <h4 class="sync-section-title" id="sync-now-title">{$t("backup.sync_section_now")}</h4>
          {#if $backupSyncState.last_synced_at && $backupSyncState.last_device}
            <p class="sync-hint" data-testid="backup-sync-last-device">{$t("backup.sync_last_from", {
              time: $formatDate($backupSyncState.last_synced_at * 1000, { dateStyle: "short", timeStyle: "short" }),
              device: $backupSyncState.last_device === $backupSyncState.this_device ? $t("backup.sync_this_device") : $backupSyncState.last_device,
            })}</p>
          {/if}
          <div class="sync-rows">
            <div class="sync-row">
              <span class="sync-row-text"><span class="sync-row-label">{$t("backup.sync_push_now")}</span><span class="sync-row-desc">{$t("backup.sync_push_desc")}</span></span>
              <button class="pill-action primary-pill" data-testid="backup-sync-push" disabled={$backupBusy} onclick={() => void runSyncAction(syncPushNow)}>{$t("backup.sync_push_button")}</button>
            </div>
            <div class="sync-row">
              <span class="sync-row-text"><span class="sync-row-label">{$t("backup.sync_pull_preview")}</span><span class="sync-row-desc">{$t("backup.sync_pull_preview_desc")}</span></span>
              <button class="pill-action" data-testid="backup-sync-pull-preview" disabled={$backupBusy} onclick={() => void runSyncAction(syncPullPreview)}>{$t("backup.sync_pull_preview_button")}</button>
            </div>
            <div class="sync-row">
              <span class="sync-row-text"><span class="sync-row-label">{$t("backup.sync_pull_direct")}</span><span class="sync-row-desc">{$t("backup.sync_pull_direct_desc")}</span></span>
              <button class="pill-action" data-testid="backup-sync-pull-direct" disabled={$backupBusy} onclick={() => void runSyncAction(() => syncPullDirect(directApplyPrefs))}>{$t("backup.sync_pull_direct_button")}</button>
            </div>
          </div>
        </section>
      {/if}

        </div>
        {#if syncProblem}
          <div class="sync-error" class:notice={linkNeeded} data-testid="backup-sync-error">
            {#if driveFix}<p class="fix" data-testid="backup-sync-fix">{$t(`backup.fix_${driveFix}` as MessageKey)}</p>{/if}
            <ErrorMessage error={syncProblem} />
          </div>
        {/if}
      </details>
      </div>
    </div>

    <div class="setting-subgroup" role="group" aria-labelledby="backup-local-heading">
      <h3 class="subgroup-title" id="backup-local-heading">{$t("backup.local_heading")}</h3>
      <p class="subgroup-desc">{$t("backup.local_description")}</p>
      <BackupLocalPanel />
    </div>

    <div class="setting-subgroup" role="group" aria-labelledby="backup-files-heading">
      <h3 class="subgroup-title" id="backup-files-heading">{$t("backup.group_files")}</h3>
      <p class="subgroup-desc">{$t("backup.group_files_description")}</p>
      <div class="backup-cards">
        <BackupCard id="backup-manual-heading" icon="download" title={$t("backup.manual_heading")} description={$t("backup.manual_description")}>
          {#snippet actions()}
            <button class="pill-action primary-pill" data-testid="backup-export" disabled={$backupBusy} onclick={saveBackup}>{$t("backup.export")}</button>
            <button class="pill-action" data-testid="backup-select" disabled={$backupBusy} onclick={selectBackup}>{$t("backup.import")}</button>
          {/snippet}
        </BackupCard>

        <BookmarkExportCard />
      </div>

      <!-- What happens after exporting or choosing a file is shown next to the buttons that started it. -->
      <BackupUnlockPrompt />
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
    </div>

  {/if}
</section>
<style>
  p { font-size: 12px; line-height: 1.6; overflow-wrap: anywhere; }
  p { color: var(--text-secondary); }
  /* Every category keeps the same readable width, so the cards of one line up with the next. */
  .backup-settings > .setting-subgroup { max-width: 960px; }
  .subgroup-desc { margin: -4px 0 2px; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 10px 0; }
  aside { border: 1px solid var(--border-subtle); border-radius: 12px; padding: 14px; margin-top: 0; background: var(--bg-glass, rgba(255, 255, 255, 0.025)); }
  /* Three cards of the same closed height. Rows do not stretch, so opening the Drive card grows only that card. */
  .backup-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(100%, 280px), 1fr)); align-items: stretch; gap: 12px; }
  /* Closed: exactly the size of its neighbours. Open: grows on every side over the page without moving anything. */
  .sync-scrim { position: fixed; inset: 0; z-index: 29; padding: 0; border: 0; background: transparent; cursor: default; }
  /* The category is the size reference of the open panel, so it can never grow past the section. */
  .sync-group { container-type: inline-size; }
  .sync-slot { --sync-closed-height: 200px; position: relative; width: min(100%, 300px); height: var(--sync-closed-height); }
  .sync-panel { position: absolute; top: 0; left: 0; width: 100%; display: block; box-sizing: border-box; height: var(--sync-closed-height); padding: 16px; overflow: hidden; border: 1px solid var(--border-subtle); border-radius: 14px; background: var(--bg-glass, rgba(255,255,255,.025)); interpolate-size: allow-keywords; transition: width 0.28s ease, height 0.28s ease, background-color 0.28s ease, box-shadow 0.28s ease; }
  /* Open: grows to the right and down from where the card is, within the width of the category. */
  .sync-panel[open] { width: min(580px, 100cqw); z-index: 30; height: auto; max-height: min(640px, 80vh); overflow-y: auto; background: var(--bg-permission-popover, #1c1c1c); box-shadow: 0 24px 64px rgba(0, 0, 0, 0.55); }
  /* Keeps the content visible while the panel shrinks back (Chromium/WebView2; elsewhere it just snaps). */
  .sync-panel::details-content { transition: content-visibility 0.28s allow-discrete; }
  @media (prefers-reduced-motion: reduce) { .sync-panel, .sync-panel::details-content, .sync-icon { transition: none; } }
  /* Closed: exactly the size of its neighbours. Open: grows on every side over the page without moving anything. */
  /* Closed: a standing block, the Drive icon on top, the mode name and its state below. */
  .sync-compact-row { display: flex; box-sizing: border-box; height: var(--sync-closed-height); flex-direction: column; align-items: center; justify-content: center; gap: 7px; padding: 2px; text-align: center; cursor: pointer; list-style: none; }
  .sync-icon { display: grid; flex: 0 0 auto; width: 56px; height: 56px; place-items: center; border: 1px solid var(--border-subtle); border-radius: 16px; background: color-mix(in srgb, var(--text-primary) 6%, transparent); color: var(--accent-primary); transition: width 0.28s ease, height 0.28s ease, border-radius 0.28s ease; }
  .sync-title { color: var(--text-primary); font-size: 14px; font-weight: 700; line-height: 1.3; }
  .sync-chips { display: flex; flex-direction: column; align-items: center; gap: 6px; max-width: 100%; }
  /* The status ring over the closed card (the slot has its size): a conic gradient cut down to the border, filled by --sync-fill (see settings.css). */
  .sync-slot::before { content: ""; position: absolute; inset: 0; z-index: 2; padding: 2px; border-radius: 14px; background: conic-gradient(var(--sync-ring-color, transparent) var(--sync-fill), transparent 0); opacity: 0; pointer-events: none; -webkit-mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0); -webkit-mask-composite: xor; mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0); mask-composite: exclude; transition: opacity 180ms ease; }
  .sync-slot:is(.ring-syncing, .ring-done, .ring-paused, .ring-error)::before { opacity: 1; }
  .sync-slot.ring-syncing { --sync-ring-color: var(--download-status-completed, #22c55e); }
  .sync-slot.ring-syncing::before { animation: sync-fill 1.6s linear infinite; }
  .sync-slot.ring-done { --sync-ring-color: var(--download-status-completed, #22c55e); }
  .sync-slot.ring-done::before { --sync-fill: 100%; animation: sync-done 320ms ease-out; }
  .sync-slot.ring-paused { --sync-ring-color: var(--download-status-paused, #eab308); }
  .sync-slot.ring-error { --sync-ring-color: var(--download-status-error, #ef4444); }
  .sync-slot:is(.ring-paused, .ring-error)::before { --sync-fill: 100%; }
  @keyframes sync-fill { from { --sync-fill: 0%; } to { --sync-fill: 100%; } }
  @keyframes sync-done { from { opacity: 0.4; } to { opacity: 1; } }
  @media (prefers-reduced-motion: reduce) { .sync-slot::before { animation: none !important; transition: none; } .sync-slot.ring-syncing::before { --sync-fill: 35%; } }
  .sync-compact-row::-webkit-details-marker, .sync-compact-row::marker { display: none; }
  .sync-compact-row:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 3px; border-radius: 8px; }
  .sync-panel[open] > .sync-compact-row { height: auto; flex-direction: row; flex-wrap: wrap; justify-content: flex-start; gap: 8px 10px; padding: 0 0 12px; text-align: left; border-bottom: 1px solid var(--border-subtle); }
  .sync-panel[open] .sync-icon { width: 40px; height: 40px; border-radius: 12px; }
  .sync-panel[open] .sync-icon :global(svg) { width: 24px; height: auto; }
  .sync-panel[open] .sync-chips { flex-direction: row; flex-wrap: wrap; }
  /* Wide panel: description and mode on the left, the mode's fields on the right, sync buttons under the mode. */
  .sync-panel { container-type: inline-size; --text-secondary: color-mix(in srgb, var(--text-primary) 82%, transparent); --sync-error-text: color-mix(in srgb, #ef4444 52%, var(--text-primary)); }
  .sync-panel .pill-action { font-size: 12.5px; }
  .sync-state-chip.warn { border-color: rgba(251, 191, 36, 0.45); background: rgba(251, 191, 36, 0.12); color: #fcd34d; }
  .sync-error .fix { margin: 0 0 8px; color: var(--text-primary); font-size: 13px; font-weight: 600; line-height: 1.55; }
  .sync-error.notice { border-color: rgba(251, 191, 36, 0.45); background: rgba(251, 191, 36, 0.08); }
  .sync-error.notice :global(.error-message) { color: color-mix(in srgb, #f59e0b 50%, var(--text-primary)); }
  .sync-error { margin-top: 12px; padding: 12px 14px; border: 1px solid rgba(248, 113, 113, 0.45); border-radius: 10px; background: rgba(248, 113, 113, 0.1); }
  .sync-error :global(.error-message) { color: var(--sync-error-text); font-size: 13px; line-height: 1.6; }
  .sync-error :global(.error-message p) { margin: 0; }
  .sync-error :global(.error-message summary) { margin-top: 8px; width: fit-content; cursor: pointer; font-size: 12.5px; font-weight: 600; text-decoration: underline; text-underline-offset: 3px; }
  .sync-error :global(.error-message pre) { margin: 8px 0 0; padding: 10px 12px; border-radius: 8px; background: rgba(0, 0, 0, 0.3); color: var(--text-primary); font-size: 12px; line-height: 1.5; }
  .sync-settings { display: flex; width: 100%; min-width: 0; flex-direction: column; gap: 16px; padding-top: 14px; }
  /* Each part of the open panel: a small title, then its options. */
  .sync-section { display: flex; min-width: 0; flex-direction: column; gap: 8px; }
  .sync-section-title { margin: 0; color: var(--text-muted, var(--text-secondary)); font-size: 11px; font-weight: 700; letter-spacing: 0.6px; text-transform: uppercase; }
  .sync-rows { display: flex; flex-direction: column; overflow: hidden; border: 1px solid var(--border-subtle); border-radius: 10px; background: rgba(255, 255, 255, 0.025); }
  .sync-row { display: flex; align-items: center; justify-content: space-between; gap: 14px; padding: 11px 12px; }
  .sync-row + .sync-row { border-top: 1px solid var(--border-subtle); }
  label.sync-row { cursor: pointer; }
  .sync-row-text { display: flex; min-width: 0; flex-direction: column; gap: 2px; }
  .sync-row-label { color: var(--text-primary); font-size: 12.5px; font-weight: 600; line-height: 1.4; }
  .sync-row-desc { color: var(--text-secondary); font-size: 11.5px; line-height: 1.5; }
  .sync-row > button { flex: 0 0 auto; min-width: 96px; }
  .sync-row-buttons { display: flex; flex: 0 0 auto; gap: 6px; }
  .sync-row-select { flex: 0 0 auto; max-width: 52%; padding: 6px 9px; border: 1px solid var(--border-subtle); border-radius: 7px; background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 12px; }
  .sync-mode-field small { color: var(--text-secondary); font-size: 11.5px; font-weight: 400; line-height: 1.5; }
  .sync-state-chip { display: inline-flex; max-width: 100%; align-items: center; justify-content: center; padding: 5px 11px; border: 1px solid var(--border-subtle); border-radius: 999px; background: rgba(255,255,255,.04); color: var(--text-secondary); font-size: 12px; line-height: 1.35; overflow-wrap: anywhere; }
  .sync-state-chip.connection.connected, .sync-state-chip.synced { border-color: rgba(34, 197, 94, .35); background: rgba(34, 197, 94, .1); color: #86efac; }
  .sync-state-chip.connection:not(.connected) { border-color: rgba(255,255,255,.12); }
  .sync-state-chip.busy { border-color: rgba(66, 133, 244, .4); background: rgba(66, 133, 244, .12); color: #93c5fd; }
  .sync-state-chip.error { border-color: rgba(248, 113, 113, .42); background: rgba(248, 113, 113, .12); color: #fca5a5; }
  .sync-state-chip.neutral { color: var(--text-secondary); }
  .sync-field { display: flex; flex-direction: column; gap: 6px; margin: 0; font-size: 12.5px; font-weight: 600; color: var(--text-primary); }
  .sync-field input, .sync-field select { min-width: 0; padding: 7px 9px; border-radius: 7px; border: 1px solid var(--border-subtle); background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 12.5px; }
  .sync-path { font-family: monospace; color: var(--text-primary); padding: 8px 10px; border-radius: 8px; background: rgba(255, 255, 255, 0.03); border: 1px solid var(--border-subtle); }
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
    margin: 0;
    font-size: 12.5px;
    line-height: 1.6;
    color: var(--text-secondary);
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
  @media (max-width: 520px) {
    aside { padding: 14px; border-radius: 12px; }
    .gdrive-account-card { align-items: flex-start; }
    .gdrive-account-actions { width: 100%; }
    .gdrive-account-actions :global(button) { max-width: 100%; white-space: normal; text-align: left; }
    .sync-row:has(> button) { flex-wrap: wrap; }
  }
</style>
