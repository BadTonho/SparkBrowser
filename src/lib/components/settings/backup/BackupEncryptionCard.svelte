<script lang="ts">
  import { t } from "../../../i18n";
  import {
    backupBusy,
    localBackupBusy,
    localBackupError,
    localBackupNotice,
    localBackupState,
    saveEncryption,
  } from "../../../stores/backupStore";
  import BackupCard from "./BackupCard.svelte";
  import BackupSwitch from "./BackupSwitch.svelte";

  let password = $state("");
  let wantsEncryption = $state(false);

  const busy = $derived($localBackupBusy || $backupBusy);
  const enabled = $derived($localBackupState.encryption_enabled);

  $effect(() => {
    wantsEncryption = $localBackupState.encryption_enabled;
  });

  function toggle(checked: boolean): void {
    wantsEncryption = checked;
    // Turning it off applies immediately; turning it on waits for a password.
    if (!checked && enabled) void saveEncryption(false);
  }
  async function savePassword(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    await saveEncryption(true, password);
    if (!$localBackupError) password = "";
  }
</script>

<BackupCard id="backup-encryption-heading" icon="lock" title={$t("backup.encrypt_heading")} description={$t("backup.encrypt_description")}>
  <form data-testid="backup-encryption" onsubmit={savePassword}>
    <BackupSwitch label={$t("backup.encrypt_enabled")} checked={wantsEncryption} disabled={busy} testid="backup-encrypt-enabled" onchange={toggle} />
    {#if wantsEncryption}
      <label class="field">
        <span>{$t("backup.encrypt_password")}</span>
        <input
          type="password"
          data-testid="backup-encrypt-password"
          bind:value={password}
          placeholder={enabled ? $t("backup.encrypt_password_saved") : $t("backup.encrypt_password_hint")}
          disabled={busy}
          autocomplete="new-password"
          minlength="8"
          maxlength="256"
          required={!enabled}
        />
      </label>
      <button class="pill-action" type="submit" data-testid="backup-encrypt-save" disabled={busy || (!password && !enabled)}>
        {$t(enabled ? "backup.encrypt_change" : "backup.encrypt_enable")}
      </button>
      {#if $localBackupNotice === "encryption_saved"}
        <p class="notice" role="status">{$t("backup.encrypt_saved")}</p>
      {/if}
      {#if enabled}<p class="hint">{$t("backup.encrypt_older_note")}</p>{/if}
    {/if}
  </form>
</BackupCard>

<style>
  form { display: flex; flex-direction: column; gap: 12px; }
  form button { align-self: flex-start; }
  .notice { margin: 0; color: #86efac; font-size: 11.5px; }
  .hint { margin: 0; color: var(--text-secondary); font-size: 11.5px; line-height: 1.55; }
</style>
