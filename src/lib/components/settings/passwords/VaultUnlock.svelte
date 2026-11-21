<script lang="ts">
  import { t } from "../../../i18n";
  import { eraseVault, unlockWithPassword, unlockWithRecoveryKey, vaultBusy } from "../../../stores/vaultStore";

  let useRecovery = $state(false);
  let secret = $state("");
  let resetting = $state(false);

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const value = secret;
    secret = "";
    if (!value) return;
    if (useRecovery) await unlockWithRecoveryKey(value);
    else await unlockWithPassword(value);
  }

  function switchMode(): void {
    useRecovery = !useRecovery;
    secret = "";
  }

  async function erase(): Promise<void> {
    resetting = false;
    await eraseVault();
  }
</script>

<div class="vault-card" data-testid="vault-unlock">
  <form class="vault-form" onsubmit={submit}>
    <h3>{$t("passwords.unlock_title")}</h3>
    <p>{$t("passwords.unlock_description")}</p>
    <label class="vault-field">
      <span>{$t(useRecovery ? "passwords.recovery_key" : "passwords.master_password")}</span>
      <input type={useRecovery ? "text" : "password"} bind:value={secret} autocomplete="off" spellcheck="false" maxlength="256"
        disabled={$vaultBusy} data-testid="vault-unlock-secret" />
    </label>
    <div class="vault-actions">
      <button class="pill-action primary-pill" type="submit" disabled={$vaultBusy || !secret} data-testid="vault-unlock-submit">{$t("passwords.unlock")}</button>
    </div>
  </form>
  <button class="vault-link" type="button" onclick={switchMode}>{$t(useRecovery ? "passwords.use_password" : "passwords.use_recovery")}</button>
  {#if !resetting}
    <button class="vault-link" type="button" onclick={() => (resetting = true)} data-testid="vault-reset-open">{$t("passwords.reset_open")}</button>
  {:else}
    <div role="alertdialog" aria-labelledby="vault-reset-title" class="vault-reset" data-testid="vault-reset">
      <h3 id="vault-reset-title">{$t("passwords.reset_title")}</h3>
      <p>{$t("passwords.reset_warning")}</p>
      <div class="vault-actions">
        <button class="pill-action" type="button" onclick={erase} disabled={$vaultBusy} data-testid="vault-reset-confirm">{$t("passwords.reset_confirm")}</button>
        <button class="pill-action" type="button" onclick={() => (resetting = false)}>{$t("passwords.cancel")}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .vault-form, .vault-reset { display: flex; flex-direction: column; gap: 10px; }
  .vault-reset h3 { font-size: 14px; }
  .vault-reset { padding-top: 6px; border-top: 1px solid var(--border-subtle); }
</style>
