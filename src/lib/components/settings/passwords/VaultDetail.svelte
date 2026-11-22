<script lang="ts">
  import { t } from "../../../i18n";
  import { avatarSlot } from "../../../services/vault/groups.js";
  import { copyEntryPassword, copyVaultSecret, vaultBusy } from "../../../stores/vaultStore";
  import type { VaultRevealed } from "../../../types/vault";
  import TotpCode from "./TotpCode.svelte";
  import VaultDialog from "./VaultDialog.svelte";
  import VaultIcon from "./VaultIcon.svelte";

  // One saved login, laid out like the password page of other browsers. It is only ever opened after the
  // person confirmed who they are, and the list closes it after five minutes away.
  let { entry, site, onclose, onedit, ondelete }: {
    entry: VaultRevealed;
    site: string;
    onclose: () => void;
    onedit: () => void;
    ondelete: () => void;
  } = $props();

  let visible = $state(false);
  let deleting = $state(false);
  /** Only success is shown here; a failure appears as the vault error message. */
  let copied = $state(false);

  async function copyPassword(): Promise<void> {
    copied = await copyEntryPassword(entry.id);
  }

  async function copyUsername(): Promise<void> {
    copied = await copyVaultSecret(entry.username);
  }
</script>

<div class="vault-detail" data-testid="vault-detail">
  <div class="vault-detail-head">
    <button class="vault-icon-button" type="button" onclick={onclose} aria-label={$t("passwords.detail_back")} data-testid="vault-detail-back"><VaultIcon name="back" /></button>
    <span class={`vault-avatar hue-${avatarSlot(site)}`} aria-hidden="true">{site.charAt(0).toUpperCase()}</span>
    <h3>{site}</h3>
    {#if entry.title.trim().toLowerCase() !== site}<span class="vault-detail-title">{entry.title}</span>{/if}
  </div>

  <div class="vault-detail-card">
    <div class="vault-detail-grid">
      <div class="vault-detail-column">
        <span class="vault-detail-label">{$t("passwords.detail_username")}</span>
        <div class="vault-detail-field">
          <span class="vault-detail-value" data-testid="vault-detail-username">{entry.username || "—"}</span>
          {#if entry.username}
            <button class="vault-icon-button" type="button" disabled={$vaultBusy} onclick={copyUsername} aria-label={$t("passwords.copy_username")}><VaultIcon name="copy" /></button>
          {/if}
        </div>
        <span class="vault-detail-label">{$t("passwords.field_password")}</span>
        <div class="vault-detail-field">
          <span class="vault-detail-value vault-detail-secret" data-testid="vault-detail-password">{visible ? entry.password : "••••••••••••"}</span>
          <button class="vault-icon-button" type="button" onclick={() => (visible = !visible)} aria-label={$t(visible ? "passwords.hide" : "passwords.show")} aria-pressed={visible} data-testid="vault-detail-eye"><VaultIcon name={visible ? "eye-off" : "eye"} /></button>
          <button class="vault-icon-button" type="button" disabled={$vaultBusy} onclick={copyPassword} aria-label={$t("passwords.copy_password")} data-testid="vault-detail-copy"><VaultIcon name="copy" /></button>
        </div>
        {#if entry.totp}
          <span class="vault-detail-label">{$t("passwords.detail_totp")}</span>
          <TotpCode secret={entry.totp} />
        {/if}
      </div>
      <div class="vault-detail-column">
        <span class="vault-detail-label">{$t("passwords.detail_sites")}</span>
        <span class="vault-detail-site">{entry.url || $t("passwords.detail_no_site")}</span>
        <span class="vault-detail-label">{$t("passwords.detail_note")}</span>
        <div class="vault-detail-note" class:empty={!entry.notes}>{entry.notes || $t("passwords.detail_no_note")}</div>
      </div>
    </div>
    {#if copied}<p class="vault-ok" role="status">{$t("passwords.copied")}</p>{/if}
    <div class="vault-detail-actions">
      <button class="pill-action" type="button" onclick={onedit} data-testid="vault-edit">{$t("passwords.edit")}</button>
      <button class="pill-action" type="button" onclick={() => (deleting = true)} data-testid="vault-delete">{$t("passwords.delete")}</button>
    </div>
  </div>
</div>

{#if deleting}
  <VaultDialog title={$t("passwords.delete_confirm")} onclose={() => (deleting = false)} testid="vault-delete-dialog">
    <div class="vault-actions">
      <button class="pill-action primary-pill" type="button" disabled={$vaultBusy} onclick={ondelete} data-testid="vault-delete-confirm">{$t("passwords.delete")}</button>
      <button class="pill-action" type="button" onclick={() => (deleting = false)}>{$t("passwords.cancel")}</button>
    </div>
  </VaultDialog>
{/if}
