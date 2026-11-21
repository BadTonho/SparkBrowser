<script lang="ts" generics="T">
  import { t } from "../../../i18n";
  import { vaultBusy, vaultFailure, vaultStatus } from "../../../stores/vaultStore";
  import VaultDialog from "./VaultDialog.svelte";

  // Opening a password asks who is there, every time: the master password or Windows Hello. The backend
  // enforces it; this dialog only collects the proof.
  // `reveal` asks the backend to open the item with the proof given (the typed password, or null for
  // Windows Hello). It works for logins and for saved cards alike.
  let { reveal, site, onconfirmed, oncancel }: {
    reveal: (secret: string | null) => Promise<T | undefined>;
    site: string;
    onconfirmed: (revealed: T) => void;
    oncancel: () => void;
  } = $props();

  let secret = $state("");

  const hasPassword = $derived($vaultStatus?.has_password !== false);
  const hasHello = $derived($vaultStatus?.has_hello === true);
  // "Use Windows Hello when showing or copying passwords" turns the typed password off here.
  const helloRequired = $derived(hasHello && $vaultStatus?.require_hello === true);
  const showPassword = $derived(hasPassword && !helloRequired);

  async function withPassword(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const value = secret;
    secret = "";
    if (!value) return;
    const revealed = await reveal(value);
    if (revealed) onconfirmed(revealed);
  }

  async function withHello(): Promise<void> {
    const revealed = await reveal(null);
    if (revealed) onconfirmed(revealed);
  }

  function cancel(): void {
    secret = "";
    vaultFailure.set(null);
    oncancel();
  }
</script>

<VaultDialog title={$t("passwords.confirm_title")} onclose={cancel} testid="vault-confirm" icon="lock">
  <p class="vault-note">{$t("passwords.confirm_text", { site })}</p>
  <div class="vault-auth-controls">
    {#if showPassword}
      <form class="vault-auth-form" onsubmit={withPassword}>
        <label class="vault-field">
          <span>{$t("passwords.master_password")}</span>
          <input type="password" bind:value={secret} autocomplete="current-password" maxlength="256" disabled={$vaultBusy} data-testid="vault-confirm-password" />
        </label>
        <button class="vault-button vault-primary" type="submit" disabled={$vaultBusy || !secret} data-testid="vault-confirm-submit">{$t("passwords.confirm_button")}</button>
      </form>
    {/if}
    {#if hasHello}
      <button class="vault-button" class:vault-primary={!showPassword} type="button" disabled={$vaultBusy} onclick={withHello} data-testid="vault-confirm-hello">{$t("passwords.confirm_with_hello")}</button>
      {#if helloRequired}<p class="vault-note">{$t("passwords.confirm_hello_required")}</p>{/if}
    {/if}
    {#if $vaultFailure}<p class="vault-error" role="alert">{$t($vaultFailure.key, $vaultFailure.values)}</p>{/if}
  </div>
  <div class="vault-confirm-footer">
    <button class="vault-link" type="button" onclick={cancel}>{$t("passwords.cancel")}</button>
  </div>
</VaultDialog>
