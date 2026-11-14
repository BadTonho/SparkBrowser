<script lang="ts">
  import { t, formatDate, formatNumber, type MessageKey } from "../../../i18n";
  import { localBackupState } from "../../../stores/backupStore";
  import { UserError, describeError } from "../../../services/user-errors.js";
  import ErrorMessage from "../../status/ErrorMessage.svelte";
  import BackupCard from "./BackupCard.svelte";
  import { formatBytes } from "./backupFormat";

  const schedule = $derived($localBackupState.schedule);
  /** Newest regular backup; "Before replace" safety copies are not what the schedule produces. */
  const lastBackup = $derived($localBackupState.entries.find((entry) => entry.reason !== "pre_restore") ?? null);
  const failure = $derived($localBackupState.last_failure);
  const failureError = $derived(failure ? describeError(new UserError(failure.code, failure.code)) : "");
  const status = $derived<"failed" | "ok" | "none">(failure ? "failed" : lastBackup ? "ok" : "none");
  const nextDue = $derived(schedule.enabled ? $localBackupState.next_due_at : null);

  const when = (seconds: number) => $formatDate(seconds * 1000, { dateStyle: "short", timeStyle: "short" });
</script>

<BackupCard id="backup-status-heading" icon="clock" title={$t("backup.status_heading")} description={$t("backup.status_description")}>
  <div class="status" data-testid="backup-status">
    <span class="chip" class:ok={status === "ok"} class:error={status === "failed"} role={status === "failed" ? "alert" : "status"} data-testid="backup-status-chip">
      <span class="dot" aria-hidden="true"></span>{$t(`backup.status_${status}` as MessageKey)}
    </span>
  </div>

  <dl class="stats">
    <div class="stat" data-testid="backup-status-last">
      <dt>{$t("backup.stat_last")}</dt>
      <dd>{lastBackup ? when(lastBackup.created_at) : $t("backup.stat_never")}</dd>
    </div>
    <div class="stat" data-testid="backup-schedule-next">
      <dt>{$t("backup.stat_next")}</dt>
      <dd>{nextDue ? when(nextDue) : $t("backup.stat_off")}</dd>
    </div>
    <div class="stat" data-testid="backup-status-usage">
      <dt>{$t("backup.stat_space")}</dt>
      <dd>{formatBytes($localBackupState.total_bytes, $formatNumber)}</dd>
      <small>{$t("backup.stat_count", { count: $formatNumber($localBackupState.entries.length) })}</small>
    </div>
  </dl>

  {#if failure}
    <div class="failure" data-testid="backup-status-failure">
      <p>
        {$t("backup.status_failed_at", { time: when(failure.at) })}
        {#if schedule.enabled} {$t("backup.status_retry")}{/if}
      </p>
      <ErrorMessage error={failureError} />
    </div>
  {/if}
</BackupCard>

<style>
  .status { display: flex; }
  .chip { display: inline-flex; align-items: center; gap: 7px; padding: 4px 11px; border: 1px solid var(--border-subtle); border-radius: 999px; background: rgba(255, 255, 255, 0.04); color: var(--text-secondary); font-size: 11.5px; font-weight: 600; line-height: 1.35; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: currentColor; opacity: 0.7; }
  .chip.ok { border-color: rgba(34, 197, 94, 0.35); background: rgba(34, 197, 94, 0.1); color: #86efac; }
  .chip.error { border-color: rgba(248, 113, 113, 0.42); background: rgba(248, 113, 113, 0.12); color: #fca5a5; }
  .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px; margin: 0; }
  .stat { display: flex; min-width: 0; flex-direction: column; gap: 3px; padding: 11px 12px; border: 1px solid var(--border-subtle); border-radius: 10px; background: rgba(255, 255, 255, 0.025); }
  dt { color: var(--text-muted, var(--text-secondary)); font-size: 10.5px; font-weight: 700; letter-spacing: 0.5px; text-transform: uppercase; }
  dd { margin: 0; color: var(--text-primary); font-size: 14px; font-weight: 600; line-height: 1.35; overflow-wrap: anywhere; }
  small { color: var(--text-secondary); font-size: 11.5px; }
  .failure { padding: 10px 12px; border: 1px solid rgba(248, 113, 113, 0.35); border-radius: 10px; background: rgba(248, 113, 113, 0.08); }
  .failure p { margin: 0 0 4px; color: var(--text-secondary); font-size: 11.5px; line-height: 1.5; }
</style>
