<script lang="ts">
  import { t } from "../../../i18n";
  import { revealEntry, vaultBusy, vaultFailure, vaultStatus } from "../../../stores/vaultStore";
  import type { VaultRevealed, VaultSummary } from "../../../types/vault";
  import VaultDialog from "./VaultDialog.svelte";

  // Opening a password asks who is there, every time: the master password or Windows Hello. The backend
  // enforces it; this dialog only collects the proof.
  let { entry, site, onconfirmed, oncancel }: {
    entry: VaultSummary;
    site: string;
    onconfirmed: (revealed: VaultRevealed) => void;
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
    const revealed = await revealEntry(entry.id, value);
    if (revealed) onconfirmed(revealed);
  }

  async function withHello(): Promise<void> {
    const revealed = await revealEntry(entry.id, null);
    if (revealed) onconfirmed(revealed);
  }

  function cancel(): void {
    secret = "";
    vaultFailure.set(null);
    oncancel();
  }
</script>

<VaultDialog title={$t("passwords.confirm_title")} onclose={cancel} testid="vault-confirm">
  <p class="vault-note">{$t("passwords.confirm_text", { site })}</p>
  {#if hasHello}
    <div class="vault-actions">
      <button class="pill-action primary-pill" type="button" disabled={$vaultBusy} onclick={withHello} data-testid="vault-confirm-hello">{$t("passwords.confirm_with_hello")}</button>
    </div>
    {#if helloRequired}<p class="vault-note">{$t("passwords.confirm_hello_required")}</p>{/if}
  {/if}
  {#if showPassword}
    {#if hasHello}<p class="vault-note">{$t("passwords.confirm_or")}</p>{/if}
    <form class="vault-confirm-form" onsubmit={withPassword}>
      <label class="vault-field">
        <span>{$t("passwords.master_password")}</span>
        <input type="password" bind:value={secret} autocomplete="current-password" maxlength="256" disabled={$vaultBusy} data-testid="vault-confirm-password" />
      </label>
      <div class="vault-actions">
        <button class="pill-action primary-pill" type="submit" disabled={$vaultBusy || !secret} data-testid="vault-confirm-submit">{$t("passwords.confirm_button")}</button>
      </div>
    </form>
  {/if}
  {#if $vaultFailure}<p class="vault-error" role="alert">{$t($vaultFailure.key, $vaultFailure.values)}</p>{/if}
  <div class="vault-actions">
    <button class="pill-action" type="button" onclick={cancel}>{$t("passwords.cancel")}</button>
  </div>
</VaultDialog>

<style>
  .vault-confirm-form { display: flex; flex-direction: column; gap: 10px; }
</style>
