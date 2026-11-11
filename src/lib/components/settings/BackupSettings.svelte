<script lang="ts">
  import { t, labelText, formatDate, formatNumber, type MessageKey } from "../../i18n";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import {
    backupBusy,
    backupError,
    backupExported,
    backupPreview,
    backupResult,
    backupSyncNotice,
    backupSyncState,
    dismissBackupPreview,
    importBackup,
    openGoogleLoginTab,
    pendingBackupPreferences,
    pickBackupSyncFolder,
    refreshBackupPreview,
    retryBackupPreferences,
    saveBackup,
    saveBackupSyncSettings,
    selectBackup,
    syncPullDirect,
    syncPullPreview,
    syncPushNow,
  } from "../../stores/backupStore";
  import { SHORTCUT_DEFINITIONS, formatShortcut } from "../../stores/shortcutsStore";
  import type { BackupKind } from "../../types/backup";
  import GoogleDriveIcon from "../icons/GoogleDriveIcon.svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";

  const desktop = isTauriRuntime();
  let applyPreferences = $state(false);
  let skippedLimit = $state(100);
  let syncExpanded = $state(false);
  let syncActionBusy = $state(false);
  let syncActionError = $state<string | null>(null);

  let selectedMode = $state<"disabled" | "google_drive_web" | "folder" | "direct_webdav">("disabled");
  let folderPath = $state("");
  let webdavUrl = $state("");
  let webdavUsername = $state("");
  let webdavSecret = $state("");
  let rememberSecret = $state(true);
  let autoSave = $state(true);
  let directApplyPrefs = $state(false);

  $effect(() => {
    void $backupPreview?.token;
    applyPreferences = false;
    skippedLimit = 100;
  });

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

  function valueLabel(value: unknown, name: string): string {
    if (name === "site_zoom" && Array.isArray(value))
      return value.map((row) => `${row.origin}: ${$formatNumber(Math.round(row.factor * 100))}%`).join(" · ") || "100%";
    if (typeof value === "number") return $formatNumber(value);
    const labels: Record<string, MessageKey> = {
      horizontal: "backup.value_horizontal",
      vertical: "backup.value_vertical",
      toolbar: "backup.value_toolbar",
      popup: "backup.value_popup",
      auto: "backup.value_auto",
      off: "backup.value_off",
      true: "backup.value_true",
      false: "backup.value_false",
      stable: "backup.value_stable",
      beta: "backup.value_beta",
      system: "common.system",
    };
    if (name === "language")
      return value === "pt-BR" ? "Português" : value === "en" ? "English" : $t("common.system");
    return labels[String(value)] ? $t(labels[String(value)]) : String(value);
  }
</script>

<section class="setting-group backup-settings" aria-labelledby="backup-heading">
  <h2 class="group-title" id="backup-heading">{$t("backup.heading")}</h2>
  {#if !desktop}<p role="status">{$t("common.desktop_required")}</p>
  {:else}
    <div class="backup-cards">
      <article class="manual-backup-card" aria-labelledby="backup-manual-heading">
        <h3 id="backup-manual-heading">{$t("backup.manual_heading")}</h3>
        <div class="actions manual-backup-actions">
          <button class="pill-action primary-pill" data-testid="backup-export" disabled={$backupBusy} onclick={saveBackup}>{$t("backup.export")}</button>
          <button class="pill-action" data-testid="backup-select" disabled={$backupBusy} onclick={selectBackup}>{$t("backup.import")}</button>
        </div>
      </article>

      <details class="sync-panel" bind:open={syncExpanded} data-testid="backup-sync-panel">
        <summary class="sync-compact-row" data-testid="backup-sync-settings-toggle" title={$t("backup.sync_settings")}>
          <GoogleDriveIcon size={38} />
          <span class="sync-state-chip connection" class:connected={Boolean($backupSyncState.connected_account)} role="status">
            {$t($backupSyncState.connected_account ? "backup.sync_connection_connected" : "backup.sync_connection_disconnected")}
          </span>
          {#if syncActionBusy}
            <span class="sync-state-chip busy" role="status">{$t("backup.sync_status_busy")}</span>
          {:else if syncActionError || $backupSyncState.last_error}
            <span class="sync-state-chip error" role="alert" title={syncActionError || $backupSyncState.last_error || undefined}>{$t("backup.sync_status_error")}</span>
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
        <ErrorMessage error={syncActionError || $backupSyncState.last_error} />
      </details>
    </div>

    <ErrorMessage error={$backupError} />
    {#if $backupExported}<p role="status">{$t("backup.exported")}</p>{/if}
    {#if $pendingBackupPreferences}
      <aside role="status"><p>{$t("backup.pending")}</p><ErrorMessage error={$pendingBackupPreferences.error} /><button data-testid="backup-retry-preferences" disabled={$backupBusy} onclick={retryBackupPreferences}>{$t("backup.retry")}</button></aside>
    {/if}
    {#if $backupPreview}
      <article data-testid="backup-preview">
        <h3>{$t("backup.preview")}</h3><p>{$t("backup.keep_current")}</p>
        <table><thead><tr><th>{$t("backup.kind")}</th><th>{$t("backup.total")}</th><th>{$t("backup.added")}</th></tr></thead><tbody>
          {#each Object.keys($backupPreview.counts) as kind}
            <tr><th>{$t(("backup.kind_" + kind) as MessageKey)}</th><td>{$formatNumber($backupPreview.counts[kind as BackupKind])}</td><td>{$formatNumber($backupPreview.added[kind as BackupKind])}</td></tr>
          {/each}
        </tbody></table>
        <details><summary>{$t("backup.skipped", { count: $formatNumber($backupPreview.skipped.length) })}</summary>
          <ul>{#each $backupPreview.skipped.slice(0, skippedLimit) as row}<li>{$t(("backup.kind_" + row.kind) as MessageKey)}: {row.key} — {$t(row.reason === "identical" ? "backup.identical" : "backup.existing")}</li>{/each}</ul>
          {#if skippedLimit < $backupPreview.skipped.length}<button onclick={() => skippedLimit += 100}>{$t("backup.more")}</button>{/if}
        </details>
        <h4>{$t("backup.preferences")}</h4>
        <dl>{#each Object.entries($backupPreview.preferences).filter(([key]) => key !== "keyboard_shortcuts") as [key, value]}<dt>{$t(("backup.pref_" + key) as MessageKey)}</dt><dd>{valueLabel(value, key)}</dd>{/each}</dl>
        <details><summary>{$t("backup.pref_keyboard_shortcuts")}</summary><dl>
          {#each Object.entries($backupPreview.preferences.keyboard_shortcuts) as [id, bindings]}<dt>{$labelText(SHORTCUT_DEFINITIONS.find(item => item.id === id)?.label ?? id)}</dt><dd>{bindings.map(binding => formatShortcut(binding, SHORTCUT_DEFINITIONS.find(item => item.id === id)?.sequence)).join(", ")}</dd>{/each}
        </dl></details>
        <label class="preference-choice"><input type="checkbox" data-testid="backup-apply-preferences" bind:checked={applyPreferences} disabled={$backupBusy || Boolean($pendingBackupPreferences)} /> {$t("backup.apply_preferences")}</label>
        <div class="actions"><button data-testid="backup-confirm" disabled={$backupBusy} onclick={() => importBackup(applyPreferences)}>{$t("backup.confirm")}</button><button disabled={$backupBusy} onclick={refreshBackupPreview}>{$t("backup.refresh_preview")}</button><button disabled={$backupBusy} onclick={dismissBackupPreview}>{$t("backup.cancel")}</button></div>
      </article>
    {/if}
    {#if $backupResult}<p role="status" data-testid="backup-result">{$t("backup.imported")}</p>{/if}
  {/if}
</section>
<style>
  p, li, dt, dd { font-size: 12px; line-height: 1.6; overflow-wrap: anywhere; }
  p { color: var(--text-secondary); }
  h3 { margin: 0; color: var(--text-primary); font-size: 16px; line-height: 1.35; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 10px 0; }
  article, aside { border: 1px solid var(--border-subtle); border-radius: 12px; padding: 14px; margin-top: 0; background: var(--bg-glass, rgba(255, 255, 255, 0.025)); }
  .backup-cards { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); align-items: start; gap: 12px; }
  .manual-backup-card { display: flex; min-width: 0; min-height: 92px; flex-direction: column; justify-content: center; gap: 10px; }
  .sync-panel { display: block; min-width: 0; min-height: 92px; padding: 14px; border: 1px solid var(--border-subtle); border-radius: 12px; background: var(--bg-glass, rgba(255,255,255,.025)); }
  .sync-compact-row { display: flex; min-height: 92px; flex-direction: column; align-items: center; justify-content: center; gap: 8px; padding: 2px; text-align: center; cursor: pointer; list-style: none; }
  .sync-compact-row::-webkit-details-marker, .sync-compact-row::marker { display: none; }
  .sync-compact-row:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 3px; border-radius: 8px; }
  .sync-panel[open] > .sync-compact-row { min-height: 0; padding-bottom: 12px; border-bottom: 1px solid var(--border-subtle); }
  .sync-settings { display: flex; width: 100%; min-width: 0; flex-direction: column; gap: 9px; padding-top: 12px; }
  .sync-state-chip { display: inline-flex; max-width: 100%; align-items: center; justify-content: center; padding: 4px 10px; border: 1px solid var(--border-subtle); border-radius: 999px; background: rgba(255,255,255,.04); color: var(--text-secondary); font-size: 11px; line-height: 1.35; overflow-wrap: anywhere; }
  .sync-state-chip.connection.connected, .sync-state-chip.synced { border-color: rgba(34, 197, 94, .35); background: rgba(34, 197, 94, .1); color: #86efac; }
  .sync-state-chip.connection:not(.connected) { border-color: rgba(255,255,255,.12); }
  .sync-state-chip.busy { border-color: rgba(66, 133, 244, .4); background: rgba(66, 133, 244, .12); color: #93c5fd; }
  .sync-state-chip.error { border-color: rgba(248, 113, 113, .42); background: rgba(248, 113, 113, .12); color: #fca5a5; }
  .sync-state-chip.neutral { color: var(--text-secondary); }
  .manual-backup-actions { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); margin: 0; }
  .manual-backup-actions button { white-space: normal; }
  .sync-panel-heading p { margin: 3px 0 0; font-size: 11.5px; }
  table { width: 100%; font-size: 12px; border-collapse: collapse; } th, td { text-align: left; padding: 8px; border-bottom: 1px solid var(--border-subtle); }
  dl { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 6px 14px; } dd { margin: 0; }
  details ul { max-height: 220px; overflow: auto; }
  .preference-choice { display: flex; align-items: center; gap: 8px; margin: 10px 0; font-size: 12.5px; color: var(--text-primary); cursor: pointer; }
  .sync-field { display: flex; flex-direction: column; gap: 5px; margin: 10px 0; font-size: 11.5px; font-weight: 600; color: var(--text-primary); }
  .sync-field input, .sync-field select { min-width: 0; padding: 7px 9px; border-radius: 7px; border: 1px solid var(--border-subtle); background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 11.5px; }
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
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-secondary);
  }
  .gdrive-account-email {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .gdrive-account-sub {
    font-size: 11.5px;
    color: var(--text-secondary);
  }
  .gdrive-account-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .sync-hint {
    margin: 2px 0;
    font-size: 11.5px;
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
  @media (max-width: 920px) {
    .backup-cards { grid-template-columns: minmax(0, 1fr); }
  }
  @media (max-width: 520px) {
    article, aside { padding: 14px; border-radius: 12px; }
    .sync-compact-row { min-height: 100px; }
    .gdrive-account-card { align-items: flex-start; }
    .gdrive-account-actions { width: 100%; }
    .gdrive-account-actions :global(button) { max-width: 100%; white-space: normal; text-align: left; }
    .sync-actions-bar > button { flex: 1 1 100%; }
  }
</style>
