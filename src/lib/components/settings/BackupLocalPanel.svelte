<script lang="ts">
  import { t, formatDate, formatNumber, type MessageKey } from "../../i18n";
  import {
    backupBusy,
    backupNow,
    localBackupBusy,
    localBackupError,
    localBackupNotice,
    localBackupState,
    removeLocalBackup,
    restoreFromHistory,
    saveEncryption,
    saveLocalSchedule,
  } from "../../stores/backupStore";
  import { UserError, describeError } from "../../services/user-errors.js";
  import type { LocalBackupInterval, LocalBackupSchedule } from "../../types/backup";
  import ErrorMessage from "../status/ErrorMessage.svelte";

  const MAX_VERSIONS = 100;
  const INTERVALS: LocalBackupInterval[] = ["six_hours", "daily", "weekly"];

  const MAX_AGE_DAYS = 3650;
  let maxBackups = $state(10);
  let maxAgeDays = $state<number | null>(null);
  let password = $state("");
  let wantsEncryption = $state(false);
  let confirmDeleteId = $state<string | null>(null);

  const busy = $derived($localBackupBusy || $backupBusy);
  const schedule = $derived($localBackupState.schedule);
  /** Newest regular backup; "Before replace" safety copies are not what the schedule produces. */
  const lastBackup = $derived($localBackupState.entries.find((entry) => entry.reason !== "pre_restore") ?? null);
  const failure = $derived($localBackupState.last_failure);
  const failureError = $derived(failure ? describeError(new UserError(failure.code, failure.code)) : "");
  const status = $derived<"failed" | "ok" | "none">(failure ? "failed" : lastBackup ? "ok" : "none");

  $effect(() => {
    maxBackups = $localBackupState.schedule.max_backups;
  });
  $effect(() => {
    maxAgeDays = $localBackupState.schedule.max_age_days;
  });
  $effect(() => {
    wantsEncryption = $localBackupState.encryption_enabled;
  });

  const when = (seconds: number) =>
    $formatDate(seconds * 1000, { dateStyle: "short", timeStyle: "short" });

  function update(patch: Partial<LocalBackupSchedule>): void {
    void saveLocalSchedule({ ...schedule, ...patch });
  }
  function commitMaxBackups(): void {
    const value = Math.min(MAX_VERSIONS, Math.max(1, Math.round(Number(maxBackups)) || schedule.max_backups));
    maxBackups = value;
    if (value !== schedule.max_backups) update({ max_backups: value });
  }
  /** Empty means "no age limit"; anything else is clamped to a whole number of days. */
  function commitMaxAge(): void {
    const raw = Number(maxAgeDays);
    const value = maxAgeDays === null || !Number.isFinite(raw) || raw <= 0 ? null : Math.min(MAX_AGE_DAYS, Math.max(1, Math.round(raw)));
    maxAgeDays = value;
    if (value !== schedule.max_age_days) update({ max_age_days: value });
  }
  function toggleEncryption(): void {
    // Turning it off applies immediately; turning it on waits for a password.
    if (!wantsEncryption && $localBackupState.encryption_enabled) void saveEncryption(false);
  }
  async function savePassword(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    await saveEncryption(true, password);
    if (!$localBackupError) password = "";
  }
  function size(bytes: number): string {
    return bytes < 1024 * 1024
      ? `${$formatNumber(Math.max(1, Math.round(bytes / 1024)))} KB`
      : `${$formatNumber(bytes / (1024 * 1024), { maximumFractionDigits: 1 })} MB`;
  }
  function remove(id: string): void {
    if (confirmDeleteId !== id) {
      confirmDeleteId = id;
      return;
    }
    confirmDeleteId = null;
    void removeLocalBackup(id);
  }
</script>

<section class="local-panel" aria-labelledby="backup-local-heading" data-testid="backup-local-panel">
  <header>
    <h3 id="backup-local-heading">{$t("backup.local_heading")}</h3>
    <p>{$t("backup.local_description")}</p>
  </header>

  <div class="block status-block" data-testid="backup-status">
    <span class="chip" class:ok={status === "ok"} class:error={status === "failed"} role={status === "failed" ? "alert" : "status"} data-testid="backup-status-chip">
      {$t(`backup.status_${status}` as MessageKey)}
    </span>
    {#if lastBackup}
      <p class="status" data-testid="backup-status-last">{$t("backup.status_last", { time: when(lastBackup.created_at) })}</p>
    {/if}
    {#if failure}
      <p class="status" data-testid="backup-status-failure">
        {$t("backup.status_failed_at", { time: when(failure.at) })}
        {#if schedule.enabled} {$t("backup.status_retry")}{/if}
      </p>
      <ErrorMessage error={failureError} />
    {/if}
    {#if schedule.enabled && $localBackupState.next_due_at}
      <p class="status" data-testid="backup-schedule-next">
        {$t("backup.schedule_next", { time: when($localBackupState.next_due_at) })}
      </p>
    {/if}
    {#if $localBackupState.entries.length > 0}
      <p class="status" data-testid="backup-status-usage">
        {$t("backup.history_summary", {
          count: $formatNumber($localBackupState.entries.length),
          size: size($localBackupState.total_bytes),
        })}
      </p>
    {/if}
  </div>

  <div class="block">
    <label class="choice">
      <input
        type="checkbox"
        data-testid="backup-schedule-enabled"
        checked={schedule.enabled}
        disabled={busy}
        onchange={(event) => update({ enabled: event.currentTarget.checked })}
      />
      <span>{$t("backup.schedule_enabled")}</span>
    </label>
    {#if schedule.enabled}
      <label class="field">
        <span>{$t("backup.schedule_interval")}</span>
        <select
          data-testid="backup-schedule-interval"
          value={schedule.interval}
          disabled={busy}
          onchange={(event) => update({ interval: event.currentTarget.value as LocalBackupInterval })}
        >
          {#each INTERVALS as interval}
            <option value={interval}>{$t(`backup.interval_${interval}` as MessageKey)}</option>
          {/each}
        </select>
      </label>
    {/if}
    <label class="choice">
      <input
        type="checkbox"
        data-testid="backup-schedule-on-exit"
        checked={schedule.on_exit}
        disabled={busy}
        onchange={(event) => update({ on_exit: event.currentTarget.checked })}
      />
      <span>{$t("backup.schedule_on_exit")}</span>
    </label>
    <label class="field">
      <span>{$t("backup.schedule_max")}</span>
      <input
        type="number"
        min="1"
        max={MAX_VERSIONS}
        data-testid="backup-schedule-max"
        bind:value={maxBackups}
        disabled={busy}
        onchange={commitMaxBackups}
      />
    </label>
    <label class="field">
      <span>{$t("backup.schedule_max_age")}</span>
      <input
        type="number"
        min="1"
        max={MAX_AGE_DAYS}
        data-testid="backup-schedule-max-age"
        bind:value={maxAgeDays}
        placeholder={$t("backup.schedule_max_age_never")}
        disabled={busy}
        onchange={commitMaxAge}
      />
    </label>
    <p class="status">{$t("backup.schedule_retention_note")}</p>
    <div class="actions">
      <button class="pill-action primary-pill" data-testid="backup-now" disabled={busy} onclick={backupNow}>
        {$t("backup.local_now")}
      </button>
      {#if $localBackupNotice === "created"}
        <span class="status ok" role="status" data-testid="backup-now-notice">{$t("backup.local_created")}</span>
      {/if}
    </div>
  </div>

  <form class="block" onsubmit={savePassword} data-testid="backup-encryption">
    <h4>{$t("backup.encrypt_heading")}</h4>
    <label class="choice">
      <input
        type="checkbox"
        data-testid="backup-encrypt-enabled"
        bind:checked={wantsEncryption}
        disabled={busy}
        onchange={toggleEncryption}
      />
      <span>{$t("backup.encrypt_enabled")}</span>
    </label>
    <p>{$t("backup.encrypt_description")}</p>
    {#if wantsEncryption}
      <label class="field">
        <span>{$t("backup.encrypt_password")}</span>
        <input
          type="password"
          data-testid="backup-encrypt-password"
          bind:value={password}
          placeholder={$localBackupState.encryption_enabled ? $t("backup.encrypt_password_saved") : $t("backup.encrypt_password_hint")}
          disabled={busy}
          autocomplete="new-password"
          minlength="8"
          maxlength="256"
          required={!$localBackupState.encryption_enabled}
        />
      </label>
      <div class="actions">
        <button class="pill-action" type="submit" data-testid="backup-encrypt-save" disabled={busy || (!password && !$localBackupState.encryption_enabled)}>
          {$t($localBackupState.encryption_enabled ? "backup.encrypt_change" : "backup.encrypt_enable")}
        </button>
        {#if $localBackupNotice === "encryption_saved"}
          <span class="status ok" role="status">{$t("backup.encrypt_saved")}</span>
        {/if}
      </div>
      {#if $localBackupState.encryption_enabled}<p>{$t("backup.encrypt_older_note")}</p>{/if}
    {/if}
  </form>

  <div class="block" data-testid="backup-history">
    <h4>{$t("backup.history_heading")}</h4>
    {#if $localBackupState.entries.length === 0}
      <p>{$t("backup.history_empty")}</p>
    {:else}
      <ul class="history">
        {#each $localBackupState.entries as entry (entry.id)}
          <li data-testid="backup-history-entry">
            <div class="entry-info">
              <strong>{when(entry.created_at)}</strong>
              <span class="status">
                {$t(`backup.history_reason_${entry.reason}` as MessageKey)} · {size(entry.size_bytes)}
                {#if entry.encrypted} · <span class="lock">{$t("backup.history_encrypted")}</span>{/if}
              </span>
            </div>
            <div class="entry-actions">
              <button class="pill-action" data-testid="backup-history-restore" disabled={busy} onclick={() => restoreFromHistory(entry.id)}>
                {$t("backup.history_restore")}
              </button>
              <button
                class="pill-action danger"
                class:armed={confirmDeleteId === entry.id}
                data-testid="backup-history-delete"
                disabled={busy}
                onclick={() => remove(entry.id)}
                onblur={() => { if (confirmDeleteId === entry.id) confirmDeleteId = null; }}
              >
                {$t(confirmDeleteId === entry.id ? "backup.history_delete_confirm" : "backup.history_delete")}
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
    {#if $localBackupState.directory}
      <p class="path">{$t("backup.history_location", { path: $localBackupState.directory })}</p>
    {/if}
  </div>

  <ErrorMessage error={$localBackupError} />
</section>

<style>
  .local-panel { display: flex; flex-direction: column; gap: 12px; margin-top: 14px; padding: 14px; border: 1px solid var(--border-subtle); border-radius: 12px; background: var(--bg-glass, rgba(255, 255, 255, 0.025)); max-width: 560px; }
  header p, .block p { margin: 3px 0 0; font-size: 12px; line-height: 1.6; color: var(--text-secondary); overflow-wrap: anywhere; }
  h3 { margin: 0; color: var(--text-primary); font-size: 16px; line-height: 1.35; }
  h4 { margin: 0; color: var(--text-primary); font-size: 13px; }
  .block { display: flex; flex-direction: column; gap: 8px; padding-top: 12px; border-top: 1px solid var(--border-subtle); }
  .choice { display: flex; align-items: center; gap: 8px; font-size: 12.5px; color: var(--text-primary); cursor: pointer; }
  .field { display: flex; flex-direction: column; gap: 5px; font-size: 11.5px; font-weight: 600; color: var(--text-primary); }
  .field input, .field select { min-width: 0; max-width: 260px; padding: 7px 9px; border-radius: 7px; border: 1px solid var(--border-subtle); background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 11.5px; }
  .actions { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
  .status { font-size: 11.5px; color: var(--text-secondary); overflow-wrap: anywhere; }
  .status.ok { color: #86efac; }
  .lock { color: #93c5fd; }
  .status-block { gap: 4px; }
  .status-block p { margin: 0; }
  .chip { align-self: flex-start; padding: 4px 10px; border: 1px solid var(--border-subtle); border-radius: 999px; background: rgba(255, 255, 255, 0.04); color: var(--text-secondary); font-size: 11px; line-height: 1.35; }
  .chip.ok { border-color: rgba(34, 197, 94, 0.35); background: rgba(34, 197, 94, 0.1); color: #86efac; }
  .chip.error { border-color: rgba(248, 113, 113, 0.42); background: rgba(248, 113, 113, 0.12); color: #fca5a5; }
  .history { display: flex; max-height: 280px; flex-direction: column; gap: 6px; margin: 0; padding: 0; overflow: auto; list-style: none; }
  .history li { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 8px; padding: 8px 10px; border: 1px solid var(--border-subtle); border-radius: 9px; background: rgba(255, 255, 255, 0.025); }
  .entry-info { display: flex; min-width: 0; flex-direction: column; gap: 2px; font-size: 12px; color: var(--text-primary); }
  .entry-actions { display: flex; gap: 6px; }
  .danger.armed { border-color: rgba(248, 113, 113, 0.5); background: rgba(248, 113, 113, 0.14); color: #fca5a5; }
  .path { font-family: monospace; }
  .primary-pill { background: rgba(66, 133, 244, 0.18); border-color: rgba(66, 133, 244, 0.42); color: var(--text-primary); font-weight: 600; }
  .primary-pill:hover:not(:disabled) { background: rgba(66, 133, 244, 0.28); }
  @media (max-width: 520px) {
    .local-panel { padding: 12px; }
    .entry-actions { width: 100%; }
    .entry-actions button { flex: 1 1 0; }
  }
</style>
