<script lang="ts">
  import { t } from "../../../i18n";
  import { VAULT_AUTO_LOCK_MINUTES } from "../../../types/vault";
  import { changeMasterPassword, chooseAutoLock, lockNow, vaultBusy, vaultNotice, vaultStatus } from "../../../stores/vaultStore";

  let current = $state("");
  let next = $state("");

  async function change(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const [old, fresh] = [current, next];
    current = "";
    next = "";
    await changeMasterPassword(old, fresh);
  }
</script>

<section class="vault-section" aria-labelledby="vault-options-heading" data-testid="vault-options">
  <h3 id="vault-options-heading">{$t("passwords.options_heading")}</h3>
  <div class="vault-actions">
    <button class="pill-action" type="button" onclick={() => void lockNow()} disabled={$vaultBusy} data-testid="vault-lock">{$t("passwords.lock_now")}</button>
  </div>
  <label class="vault-field" style="max-width: 260px; margin-top: 12px;">
    <span>{$t("passwords.auto_lock")}</span>
    <select value={$vaultStatus?.auto_lock_minutes ?? 5} disabled={$vaultBusy} data-testid="vault-auto-lock"
      onchange={event => void chooseAutoLock(Number(event.currentTarget.value))}>
      {#each VAULT_AUTO_LOCK_MINUTES as minutes (minutes)}
        <option value={minutes}>{$t("passwords.minutes", { count: minutes })}</option>
      {/each}
    </select>
  </label>
  <form class="vault-card" onsubmit={change} data-testid="vault-change-password">
    <h3>{$t("passwords.change_password")}</h3>
    <label class="vault-field">
      <span>{$t("passwords.current_password")}</span>
      <input type="password" bind:value={current} autocomplete="current-password" maxlength="256" disabled={$vaultBusy} required />
    </label>
    <label class="vault-field">
      <span>{$t("passwords.new_password")}</span>
      <input type="password" bind:value={next} autocomplete="new-password" minlength="8" maxlength="256" placeholder={$t("passwords.password_hint")} disabled={$vaultBusy} required />
    </label>
    <div class="vault-actions">
      <button class="pill-action" type="submit" disabled={$vaultBusy || !current || next.length < 8}>{$t("passwords.change_password")}</button>
    </div>
    {#if $vaultNotice === "password_changed"}<p class="vault-ok" role="status">{$t("passwords.password_changed")}</p>{/if}
  </form>
</section>
