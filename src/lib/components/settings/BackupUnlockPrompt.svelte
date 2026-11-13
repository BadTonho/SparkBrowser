<script lang="ts">
  import { t } from "../../i18n";
  import {
    backupBusy,
    backupLocked,
    cancelBackupPasswordPrompt,
    unlockBackup,
  } from "../../stores/backupStore";

  let password = $state("");

  $effect(() => {
    // A new prompt (or a closed one) never inherits a previous attempt.
    void $backupLocked;
    password = "";
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    const value = password;
    password = "";
    if (value) void unlockBackup(value);
  }
</script>

{#if $backupLocked}
  <form class="unlock" onsubmit={submit} data-testid="backup-unlock">
    <h3>{$t("backup.unlock_title")}</h3>
    <p>{$t("backup.unlock_description")}</p>
    <input
      type="password"
      data-testid="backup-unlock-password"
      bind:value={password}
      autocomplete="off"
      maxlength="256"
      aria-label={$t("backup.encrypt_password")}
      aria-invalid={$backupLocked.wrong}
      disabled={$backupBusy}
    />
    {#if $backupLocked.wrong}
      <p class="wrong" role="alert">{$t("backup.unlock_wrong")}</p>
    {/if}
    <div class="actions">
      <button class="pill-action primary-pill" type="submit" data-testid="backup-unlock-submit" disabled={$backupBusy || !password}>
        {$t("backup.unlock_submit")}
      </button>
      <button class="pill-action" type="button" disabled={$backupBusy} onclick={cancelBackupPasswordPrompt}>
        {$t("backup.cancel")}
      </button>
    </div>
  </form>
{/if}

<style>
  .unlock { display: flex; flex-direction: column; gap: 8px; margin-top: 14px; padding: 14px; max-width: 360px; border: 1px solid var(--border-subtle); border-radius: 12px; background: var(--bg-glass, rgba(255, 255, 255, 0.025)); }
  h3 { margin: 0; color: var(--text-primary); font-size: 16px; line-height: 1.35; }
  p { margin: 0; font-size: 12px; line-height: 1.6; color: var(--text-secondary); }
  .wrong { color: #fca5a5; }
  input { min-width: 0; padding: 7px 9px; border-radius: 7px; border: 1px solid var(--border-subtle); background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 12px; }
  input[aria-invalid="true"] { border-color: rgba(248, 113, 113, 0.55); }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; }
  .primary-pill { background: rgba(66, 133, 244, 0.18); border-color: rgba(66, 133, 244, 0.42); color: var(--text-primary); font-weight: 600; }
  .primary-pill:hover:not(:disabled) { background: rgba(66, 133, 244, 0.28); }
</style>
