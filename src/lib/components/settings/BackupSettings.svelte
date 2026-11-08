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
  import ErrorMessage from "../status/ErrorMessage.svelte";

  const desktop = isTauriRuntime();
  let applyPreferences = $state(false);
  let skippedLimit = $state(100);

  let selectedMode = $state<"disabled" | "folder" | "direct_webdav">("disabled");
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
    if (cfg.mode === "folder") {
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
    const next = (event.currentTarget as HTMLSelectElement).value as "disabled" | "folder" | "direct_webdav";
    selectedMode = next;
    if (next === "disabled") {
      void saveBackupSyncSettings({ mode: "disabled" }, null);
    }
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
    <p>{$t("backup.description")}</p><p>{$t("backup.scope")}</p><p>{$t("backup.exclusions")}</p><p>{$t("backup.limit")}</p>
    <div class="actions">
      <button class="pill-action" data-testid="backup-export" disabled={$backupBusy} onclick={saveBackup}>{$t("backup.export")}</button>
      <button class="pill-action" data-testid="backup-select" disabled={$backupBusy} onclick={selectBackup}>{$t("backup.import")}</button>
    </div>

    <article class="sync-panel" data-testid="backup-sync-panel">
      <h3>{$t("backup.sync_heading")}</h3>
      <p>{$t("backup.sync_description")}</p>

      <label class="sync-field">
        <span>{$t("backup.sync_mode_label")}</span>
        <select
          data-testid="backup-sync-mode"
          value={selectedMode}
          disabled={$backupBusy}
          onchange={handleModeChange}
        >
          <option value="disabled">{$t("backup.sync_mode_disabled")}</option>
          <option value="folder">{$t("backup.sync_mode_folder")}</option>
          <option value="direct_webdav">{$t("backup.sync_mode_webdav")}</option>
        </select>
      </label>

      {#if selectedMode === "folder"}
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
        <div class="actions">
          <button
            class="pill-action"
            data-testid="backup-sync-push"
            disabled={$backupBusy}
            onclick={syncPushNow}
          >
            {$t("backup.sync_push_now")}
          </button>
          <button
            class="pill-action"
            data-testid="backup-sync-pull-preview"
            disabled={$backupBusy}
            onclick={syncPullPreview}
          >
            {$t("backup.sync_pull_preview")}
          </button>
          <button
            class="pill-action"
            data-testid="backup-sync-pull-direct"
            disabled={$backupBusy}
            onclick={() => syncPullDirect(directApplyPrefs)}
          >
            {$t("backup.sync_pull_direct")}
          </button>
        </div>
      {/if}

      {#if $backupSyncState.last_synced_at}
        <p role="status" data-testid="backup-sync-last-time">
          {$t("backup.sync_last_synced", {
            time: $formatDate($backupSyncState.last_synced_at * 1000, {
              dateStyle: "short",
              timeStyle: "medium",
            }),
          })}
        </p>
      {/if}
      {#if $backupSyncNotice === "pushed"}
        <p role="status" data-testid="backup-sync-notice">{$t("backup.sync_pushed")}</p>
      {:else if $backupSyncNotice === "unchanged"}
        <p role="status" data-testid="backup-sync-notice">{$t("backup.sync_unchanged")}</p>
      {:else if $backupSyncNotice === "config_saved"}
        <p role="status" data-testid="backup-sync-notice">{$t("backup.sync_config_saved")}</p>
      {/if}
      <ErrorMessage error={$backupSyncState.last_error} />
    </article>

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
  .actions { display: flex; flex-wrap: wrap; gap: 10px; margin: 12px 0; }
  article, aside { border: 1px solid var(--border-subtle); border-radius: 10px; padding: 14px; margin-top: 12px; }
  table { width: 100%; font-size: 12px; border-collapse: collapse; } th, td { text-align: left; padding: 8px; border-bottom: 1px solid var(--border-subtle); }
  dl { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 6px 14px; } dd { margin: 0; }
  details ul { max-height: 220px; overflow: auto; }
  .preference-choice { display: block; margin: 12px 0; font-size: 13px; }
  .sync-field { display: flex; flex-direction: column; gap: 6px; margin: 10px 0; font-size: 12px; }
  .sync-field input, .sync-field select { padding: 8px 10px; border-radius: 8px; border: 1px solid var(--border-subtle); background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 12px; }
  .sync-path { font-family: monospace; color: var(--text-primary); }
</style>
