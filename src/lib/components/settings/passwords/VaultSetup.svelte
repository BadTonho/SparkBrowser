<script lang="ts">
  import { t } from "../../../i18n";
  import { createNewVault, vaultBusy, vaultStatus } from "../../../stores/vaultStore";

  let usePassword = $state(true);
  let useHello = $state(false);
  let password = $state("");
  let confirmation = $state("");
  let mismatch = $state(false);

  const helloAvailable = $derived($vaultStatus?.hello_available === true);
  const noMethod = $derived(!usePassword && !(useHello && helloAvailable));
  const passwordOk = $derived(!usePassword || password.length >= 8);

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    mismatch = usePassword && password !== confirmation;
    if (mismatch || noMethod) return;
    const value = usePassword ? password : null;
    const hello = useHello && helloAvailable;
    await createNewVault(value, hello);
    password = "";
    confirmation = "";
  }
</script>

<form class="vault-card" onsubmit={submit} data-testid="vault-setup">
  <h3>{$t("passwords.setup_title")}</h3>
  <p>{$t("passwords.setup_description")}</p>
  <div class="vault-methods" role="group" aria-labelledby="vault-methods-label">
    <span id="vault-methods-label" class="vault-methods-label">{$t("passwords.setup_methods")}</span>
    <label class="vault-choice">
      <input type="checkbox" bind:checked={usePassword} disabled={$vaultBusy} data-testid="vault-method-password" />
      <span>{$t("passwords.method_password")}</span>
    </label>
    {#if helloAvailable}
      <label class="vault-choice">
        <input type="checkbox" bind:checked={useHello} disabled={$vaultBusy} data-testid="vault-method-hello" />
        <span>{$t("passwords.method_hello")}<small>{$t("passwords.method_hello_hint")}</small></span>
      </label>
    {/if}
  </div>
  {#if usePassword}
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
  {:else if useHello && helloAvailable}
    <p class="vault-note" data-testid="vault-hello-only-warning">{$t("passwords.hello_only_warning")}</p>
  {/if}
  {#if mismatch}<p class="vault-error" role="alert">{$t("passwords.passwords_differ")}</p>{/if}
  {#if noMethod}<p class="vault-error" role="alert">{$t("passwords.method_none")}</p>{/if}
  <div class="vault-actions">
    <button class="pill-action primary-pill" type="submit" disabled={$vaultBusy || noMethod || !passwordOk} data-testid="vault-create">{$t("passwords.create_vault")}</button>
  </div>
</form>

<style>
  .vault-methods { display: flex; flex-direction: column; gap: 8px; min-width: 0; }
  .vault-methods-label { margin-bottom: 4px; color: var(--text-secondary); font-size: 12px; }
  .vault-choice { display: flex; align-items: flex-start; gap: 8px; color: var(--text-primary); font-size: 13px; }
  .vault-choice small { display: block; margin-top: 2px; color: var(--text-secondary); font-size: 11.5px; line-height: 1.5; }
  .vault-choice input { margin-top: 2px; }
</style>
