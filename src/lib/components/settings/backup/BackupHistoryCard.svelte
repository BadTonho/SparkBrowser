<script lang="ts">
  import { t, formatDate, formatNumber, type MessageKey } from "../../../i18n";
  import {
    backupBusy,
    localBackupBusy,
    localBackupState,
    removeLocalBackup,
    restoreFromHistory,
  } from "../../../stores/backupStore";
  import BackupCard from "./BackupCard.svelte";
  import { formatBytes } from "./backupFormat";

  let confirmDeleteId = $state<string | null>(null);

  const busy = $derived($localBackupBusy || $backupBusy);
  const when = (seconds: number) => $formatDate(seconds * 1000, { dateStyle: "short", timeStyle: "short" });

  function remove(id: string): void {
    if (confirmDeleteId !== id) {
      confirmDeleteId = id;
      return;
    }
    confirmDeleteId = null;
    void removeLocalBackup(id);
  }
</script>

<BackupCard id="backup-history-heading" icon="box" title={$t("backup.history_heading")} description={$t("backup.history_description")}>
  <div data-testid="backup-history">
    {#if $localBackupState.entries.length === 0}
      <p class="empty">{$t("backup.history_empty")}</p>
    {:else}
      <ul class="history">
        {#each $localBackupState.entries as entry (entry.id)}
          <li data-testid="backup-history-entry">
            <div class="info">
              <strong>{when(entry.created_at)}</strong>
              <span class="meta">
                <span class="badge" class:safety={entry.reason === "pre_restore"}>{$t(`backup.history_reason_${entry.reason}` as MessageKey)}</span>
                <span>{formatBytes(entry.size_bytes, $formatNumber)}</span>
                {#if entry.encrypted}<span class="lock">{$t("backup.history_encrypted")}</span>{/if}
              </span>
            </div>
            <div class="row-actions">
              <button class="pill-action" data-testid="backup-history-restore" disabled={busy} onclick={() => restoreFromHistory(entry.id)}>{$t("backup.history_restore")}</button>
              <button
                class="pill-action danger"
                class:armed={confirmDeleteId === entry.id}
                data-testid="backup-history-delete"
                disabled={busy}
                onclick={() => remove(entry.id)}
                onblur={() => { if (confirmDeleteId === entry.id) confirmDeleteId = null; }}
              >{$t(confirmDeleteId === entry.id ? "backup.history_delete_confirm" : "backup.history_delete")}</button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
    {#if $localBackupState.directory}
      <p class="path">{$t("backup.history_location", { path: $localBackupState.directory })}</p>
    {/if}
  </div>
</BackupCard>

<style>
  .empty { margin: 0; padding: 14px; border: 1px dashed var(--border-subtle); border-radius: 10px; color: var(--text-secondary); font-size: 12px; text-align: center; }
  .history { display: flex; max-height: 300px; flex-direction: column; gap: 8px; margin: 0; padding: 0; overflow: auto; list-style: none; }
  li { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 10px; padding: 10px 12px; border: 1px solid var(--border-subtle); border-radius: 10px; background: rgba(255, 255, 255, 0.025); }
  .info { display: flex; min-width: 0; flex-direction: column; gap: 4px; }
  strong { color: var(--text-primary); font-size: 12.5px; font-weight: 600; }
  .meta { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; color: var(--text-secondary); font-size: 11.5px; }
  .badge { padding: 1px 8px; border: 1px solid var(--border-subtle); border-radius: 999px; background: rgba(255, 255, 255, 0.05); font-weight: 600; }
  .badge.safety { border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1); color: #fcd34d; }
  .lock { color: #93c5fd; }
  .row-actions { display: grid; grid-template-columns: repeat(2, minmax(84px, 1fr)); gap: 6px; }
  .row-actions button { min-width: 0; padding: 6px 10px; overflow: hidden; text-align: center; text-overflow: ellipsis; white-space: nowrap; }
  .danger.armed { border-color: rgba(248, 113, 113, 0.5); background: rgba(248, 113, 113, 0.14); color: #fca5a5; }
  .path { margin: 10px 0 0; color: var(--text-secondary); font-family: monospace; font-size: 11px; overflow-wrap: anywhere; }
  @media (max-width: 520px) { .row-actions { width: 100%; } }
</style>
