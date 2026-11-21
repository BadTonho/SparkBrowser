<script lang="ts">
  import { t } from "../../../i18n";
  import { VAULT_AUTO_LOCK_MINUTES } from "../../../types/vault";
  import {
    changeMasterPassword, chooseAutoLock, lockNow, turnHelloOff, turnHelloOn, turnPasswordOff, turnPasswordOn, vaultBusy, vaultNotice,
    vaultStatus,
  } from "../../../stores/vaultStore";

  let current = $state("");
  let next = $state("");
  /** The inline form of the master password button: turning it off asks for the current one, on asks for a new one. */
  let passwordForm = $state<"on" | "off" | null>(null);
  let formSecret = $state("");

  const hasPassword = $derived($vaultStatus?.has_password !== false);
  const hasHello = $derived($vaultStatus?.has_hello === true);
  const helloShown = $derived(hasHello || $vaultStatus?.hello_available === true);

  async function change(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const [old, fresh] = [current, next];
    current = "";
    next = "";
    await changeMasterPassword(old, fresh);
  }

  async function submitPasswordForm(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const [mode, value] = [passwordForm, formSecret];
    formSecret = "";
    if (!mode || !value) return;
    const done = mode === "on" ? await turnPasswordOn(value) : await turnPasswordOff(value);
    if (done) passwordForm = null;
  }

  function openPasswordForm(mode: "on" | "off"): void {
    formSecret = "";
    passwordForm = passwordForm === mode ? null : mode;
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

  <div class="vault-card" data-testid="vault-methods">
    <h3>{$t("passwords.methods_heading")}</h3>
    <p>{$t("passwords.methods_note")}</p>
    <p role="status">{$t(hasPassword ? "passwords.password_status_on" : "passwords.password_status_off")}</p>
    <div class="vault-actions">
      {#if hasPassword}
        <button class="pill-action" type="button" onclick={() => openPasswordForm("off")} disabled={$vaultBusy || !hasHello} data-testid="vault-password-off">{$t("passwords.password_turn_off")}</button>
      {:else}
        <button class="pill-action" type="button" onclick={() => openPasswordForm("on")} disabled={$vaultBusy} data-testid="vault-password-on">{$t("passwords.password_turn_on")}</button>
      {/if}
    </div>
    {#if passwordForm}
      <form class="vault-form" onsubmit={submitPasswordForm} data-testid="vault-password-form">
        <label class="vault-field">
          <span>{$t(passwordForm === "on" ? "passwords.new_password" : "passwords.current_password")}</span>
          <input type="password" bind:value={formSecret} autocomplete={passwordForm === "on" ? "new-password" : "current-password"}
            minlength={passwordForm === "on" ? 8 : undefined} maxlength="256"
            placeholder={passwordForm === "on" ? $t("passwords.password_hint") : undefined} disabled={$vaultBusy} required />
        </label>
        <div class="vault-actions">
          <button class="pill-action" type="submit" disabled={$vaultBusy || !formSecret}>{$t(passwordForm === "on" ? "passwords.password_confirm_on" : "passwords.password_confirm_off")}</button>
        </div>
      </form>
    {/if}
    {#if helloShown}
      <p role="status">{$t(hasHello ? "passwords.hello_status_on" : "passwords.hello_status_off")}</p>
      <div class="vault-actions">
        {#if hasHello}
          <button class="pill-action" type="button" onclick={() => void turnHelloOff()} disabled={$vaultBusy || !hasPassword} data-testid="vault-hello-off">{$t("passwords.hello_turn_off")}</button>
        {:else}
          <button class="pill-action" type="button" onclick={() => void turnHelloOn()} disabled={$vaultBusy} data-testid="vault-hello-on">{$t("passwords.hello_turn_on")}</button>
        {/if}
      </div>
    {/if}
    {#if $vaultNotice === "hello_on"}<p class="vault-ok" role="status">{$t("passwords.notice_hello_on")}</p>{/if}
    {#if $vaultNotice === "hello_off"}<p class="vault-ok" role="status">{$t("passwords.notice_hello_off")}</p>{/if}
    {#if $vaultNotice === "password_on"}<p class="vault-ok" role="status">{$t("passwords.notice_password_on")}</p>{/if}
    {#if $vaultNotice === "password_off"}<p class="vault-ok" role="status">{$t("passwords.notice_password_off")}</p>{/if}
  </div>

  {#if hasPassword}
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
  {/if}
</section>

<style>
  .vault-form { display: flex; flex-direction: column; gap: 10px; }
</style>
