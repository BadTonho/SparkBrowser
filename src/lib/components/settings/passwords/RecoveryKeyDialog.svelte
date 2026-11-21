<script lang="ts">
  import { t } from "../../../i18n";
  import { copyVaultSecret, dismissRecoveryKey, saveRecoveryKeyFile, vaultBusy, vaultRecoveryKey } from "../../../stores/vaultStore";

  /** Only success is shown here; a failure appears as the vault error message. */
  let copied = $state(false);
  let saved = $state(false);

  async function copy(): Promise<void> {
    if ($vaultRecoveryKey) copied = await copyVaultSecret($vaultRecoveryKey);
  }

  async function save(): Promise<void> {
    if ($vaultRecoveryKey) saved = await saveRecoveryKeyFile($vaultRecoveryKey);
  }
</script>

{#if $vaultRecoveryKey}
  <div class="vault-card" role="alertdialog" aria-labelledby="vault-recovery-title" data-testid="vault-recovery">
    <h3 id="vault-recovery-title">{$t("passwords.recovery_title")}</h3>
    <p>{$t("passwords.recovery_text")}</p>
    <code class="vault-secret" data-testid="vault-recovery-key">{$vaultRecoveryKey}</code>
    <div class="vault-actions">
      <button class="pill-action" type="button" disabled={$vaultBusy} onclick={copy}>{$t("passwords.recovery_copy")}</button>
      <button class="pill-action" type="button" disabled={$vaultBusy} onclick={save} data-testid="vault-recovery-download">{$t("passwords.recovery_download")}</button>
      <button class="pill-action primary-pill" type="button" onclick={dismissRecoveryKey} data-testid="vault-recovery-confirm">{$t("passwords.recovery_confirm")}</button>
    </div>
    <p class="vault-note">{$t("passwords.recovery_download_note")}</p>
    {#if copied}<p class="vault-ok" role="status">{$t("passwords.copied")}</p>{/if}
    {#if saved}<p class="vault-ok" role="status" data-testid="vault-recovery-saved">{$t("passwords.recovery_saved")}</p>{/if}
  </div>
{/if}
