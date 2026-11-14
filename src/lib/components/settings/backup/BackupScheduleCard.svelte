<script lang="ts">
  import { t, type MessageKey } from "../../../i18n";
  import {
    backupBusy,
    backupNow,
    localBackupBusy,
    localBackupNotice,
    localBackupState,
    saveLocalSchedule,
  } from "../../../stores/backupStore";
  import type { LocalBackupInterval, LocalBackupSchedule } from "../../../types/backup";
  import BackupCard from "./BackupCard.svelte";
  import BackupSwitch from "./BackupSwitch.svelte";

  const MAX_VERSIONS = 100;
  const MAX_AGE_DAYS = 3650;
  const INTERVALS: LocalBackupInterval[] = ["six_hours", "daily", "weekly"];

  let maxBackups = $state(10);
  let maxAgeDays = $state<number | null>(null);

  const busy = $derived($localBackupBusy || $backupBusy);
  const schedule = $derived($localBackupState.schedule);

  $effect(() => {
    maxBackups = $localBackupState.schedule.max_backups;
  });
  $effect(() => {
    maxAgeDays = $localBackupState.schedule.max_age_days;
  });

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
</script>

<BackupCard id="backup-schedule-heading" icon="refresh" title={$t("backup.schedule_heading")} description={$t("backup.schedule_description")}>
  <BackupSwitch label={$t("backup.schedule_enabled")} checked={schedule.enabled} disabled={busy} testid="backup-schedule-enabled" onchange={(enabled) => update({ enabled })} />
  {#if schedule.enabled}
    <label class="field">
      <span>{$t("backup.schedule_interval")}</span>
      <select data-testid="backup-schedule-interval" value={schedule.interval} disabled={busy} onchange={(event) => update({ interval: event.currentTarget.value as LocalBackupInterval })}>
        {#each INTERVALS as interval}
          <option value={interval}>{$t(`backup.interval_${interval}` as MessageKey)}</option>
        {/each}
      </select>
    </label>
  {/if}
  <BackupSwitch label={$t("backup.schedule_on_exit")} checked={schedule.on_exit} disabled={busy} testid="backup-schedule-on-exit" onchange={(on_exit) => update({ on_exit })} />

  <div class="pair">
    <label class="field">
      <span>{$t("backup.schedule_max")}</span>
      <input type="number" min="1" max={MAX_VERSIONS} data-testid="backup-schedule-max" bind:value={maxBackups} disabled={busy} onchange={commitMaxBackups} />
    </label>
    <label class="field">
      <span>{$t("backup.schedule_max_age")}</span>
      <input type="number" min="1" max={MAX_AGE_DAYS} data-testid="backup-schedule-max-age" bind:value={maxAgeDays} placeholder={$t("backup.schedule_max_age_never")} disabled={busy} onchange={commitMaxAge} />
    </label>
  </div>
  <p class="hint">{$t("backup.schedule_retention_note")}</p>
  {#if $localBackupNotice === "created"}
    <p class="notice" role="status" data-testid="backup-now-notice">{$t("backup.local_created")}</p>
  {/if}

  {#snippet actions()}
    <button class="pill-action primary-pill" data-testid="backup-now" disabled={busy} onclick={backupNow}>{$t("backup.local_now")}</button>
  {/snippet}
</BackupCard>

<style>
  .notice { margin: 0; color: #86efac; font-size: 11.5px; }
</style>
