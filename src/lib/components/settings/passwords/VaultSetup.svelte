<script lang="ts">
  import { t } from "../../../i18n";
  import { createNewVault, vaultBusy } from "../../../stores/vaultStore";

  let password = $state("");
  let confirmation = $state("");
  let mismatch = $state(false);

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    mismatch = password !== confirmation;
    if (mismatch) return;
    const value = password;
    await createNewVault(value);
    password = "";
    confirmation = "";
  }
</script>

<form class="vault-card" onsubmit={submit} data-testid="vault-setup">
  <h3>{$t("passwords.setup_title")}</h3>
  <p>{$t("passwords.setup_description")}</p>
  <label class="vault-field">
    <span>{$t("passwords.master_password")}</span>
    <input type="password" bind:value={password} autocomplete="new-password" minlength="8" maxlength="256"
      placeholder={$t("passwords.password_hint")} disabled={$vaultBusy} required data-testid="vault-setup-password" />
  </label>
  <label class="vault-field">
    <span>{$t("passwords.confirm_password")}</span>
    <input type="password" bind:value={confirmation} autocomplete="new-password" maxlength="256"
      aria-invalid={mismatch} disabled={$vaultBusy} required data-testid="vault-setup-confirm" />
  </label>
  {#if mismatch}<p class="vault-error" role="alert">{$t("passwords.passwords_differ")}</p>{/if}
  <div class="vault-actions">
    <button class="pill-action primary-pill" type="submit" disabled={$vaultBusy || password.length < 8} data-testid="vault-create">{$t("passwords.create_vault")}</button>
  </div>
</form>
