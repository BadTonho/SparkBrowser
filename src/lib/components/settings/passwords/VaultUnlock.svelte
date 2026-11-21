<script lang="ts">
  import type { Snippet } from "svelte";
  import { t } from "../../../i18n";
  import { eraseVault, unlockWithHello, unlockWithPassword, unlockWithRecoveryKey, vaultBusy, vaultStatus } from "../../../stores/vaultStore";
  import VaultIcon from "./VaultIcon.svelte";

  let { messages }: { messages: Snippet } = $props();

  let useRecovery = $state(false);
  let secret = $state("");
  let resetting = $state(false);

  const hasPassword = $derived($vaultStatus?.has_password !== false);
  const hasHello = $derived($vaultStatus?.has_hello === true);
  // The secret form takes the master password, or the recovery key when the person switched to it
  // (or when the vault has no master password).
  const recoveryForm = $derived(useRecovery || !hasPassword);
  const showSecretForm = $derived(hasPassword || useRecovery || !hasHello);

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const value = secret;
    secret = "";
    if (!value) return;
    if (recoveryForm) await unlockWithRecoveryKey(value);
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

<section class="vault-card vault-unlock" aria-labelledby="vault-unlock-title" data-testid="vault-unlock">
  <header class="vault-unlock-header">
    <span class="vault-auth-icon"><VaultIcon name="lock" /></span>
    <h3 id="vault-unlock-title">{$t("passwords.unlock_title")}</h3>
    <p>{$t("passwords.unlock_description")}</p>
  </header>
  <div class="vault-auth-controls">
    {#if showSecretForm}
      <form class="vault-auth-form" onsubmit={submit}>
        <label class="vault-field">
          <span>{$t(recoveryForm ? "passwords.recovery_key" : "passwords.master_password")}</span>
          <input type={recoveryForm ? "text" : "password"} bind:value={secret} autocomplete="off" spellcheck="false" maxlength="256"
            disabled={$vaultBusy} data-testid="vault-unlock-secret" />
        </label>
        <button class="vault-button vault-primary" type="submit" disabled={$vaultBusy || !secret} data-testid="vault-unlock-submit">{$t("passwords.unlock")}</button>
      </form>
    {/if}
    {#if hasHello && !useRecovery}
      <button class="vault-button" class:vault-primary={!showSecretForm} type="button"
        onclick={() => void unlockWithHello()} disabled={$vaultBusy} data-testid="vault-unlock-hello">{$t("passwords.unlock_hello")}</button>
    {/if}
    {@render messages()}
  </div>
  <div class="vault-unlock-footer">
    {#if hasPassword || hasHello}
      <button class="vault-link" type="button" onclick={switchMode}>
        {$t(!useRecovery ? "passwords.use_recovery" : hasPassword ? "passwords.use_password" : "passwords.use_hello")}
      </button>
    {/if}
    {#if !resetting}
      <button class="vault-link" type="button" onclick={() => (resetting = true)} data-testid="vault-reset-open">{$t("passwords.reset_open")}</button>
    {/if}
  </div>
  {#if resetting}
    <div role="alertdialog" aria-labelledby="vault-reset-title" class="vault-reset" data-testid="vault-reset">
      <h3 id="vault-reset-title">{$t("passwords.reset_title")}</h3>
      <p>{$t("passwords.reset_warning")}</p>
      <div class="vault-actions">
        <button class="pill-action" type="button" onclick={erase} disabled={$vaultBusy} data-testid="vault-reset-confirm">{$t("passwords.reset_confirm")}</button>
        <button class="pill-action" type="button" onclick={() => (resetting = false)}>{$t("passwords.cancel")}</button>
      </div>
    </div>
  {/if}
</section>

<style>
  .vault-reset { display: flex; flex-direction: column; gap: 10px; }
  .vault-reset h3 { font-size: 14px; }
  .vault-reset { padding-top: 6px; border-top: 1px solid var(--border-subtle); }
</style>
