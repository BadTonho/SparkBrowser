<script lang="ts">
  import { get } from "svelte/store";
  import { t } from "../../../i18n";
  import { VAULT_AUTO_LOCK_MINUTES } from "../../../types/vault";
  import {
    changeMasterPassword, chooseAutoLock, eraseVault, exportPasswords, importPasswords, lockNow, turnHelloOff, turnHelloOn,
    turnPasswordOff, turnPasswordOn, turnRequireHello, vaultBusy, vaultInfo, vaultNotice, vaultStatus,
  } from "../../../stores/vaultStore";

  /** Which row has its extra form open. Only one at a time, so the page stays short. */
  let panel = $state<"password_on" | "password_off" | "change" | "export" | "delete" | null>(null);
  let formSecret = $state("");
  let current = $state("");
  let next = $state("");

  const hasPassword = $derived($vaultStatus?.has_password !== false);
  const hasHello = $derived($vaultStatus?.has_hello === true);
  const helloShown = $derived(hasHello || $vaultStatus?.hello_available === true);

  function open(which: typeof panel): void {
    formSecret = "";
    current = "";
    next = "";
    panel = panel === which ? null : which;
  }

  async function toggleRequireHello(event: Event): Promise<void> {
    const box = event.currentTarget as HTMLInputElement;
    await turnRequireHello(box.checked);
    // The backend decides: a cancelled Windows prompt leaves the setting as it was.
    box.checked = get(vaultStatus)?.require_hello === true;
  }

  async function submitPassword(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const [mode, value] = [panel, formSecret];
    formSecret = "";
    if (!value || (mode !== "password_on" && mode !== "password_off")) return;
    const done = mode === "password_on" ? await turnPasswordOn(value) : await turnPasswordOff(value);
    if (done) panel = null;
  }

  async function submitChange(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const [old, fresh] = [current, next];
    current = "";
    next = "";
    if (await changeMasterPassword(old, fresh)) panel = null;
  }

  /** Windows Hello confirms the export when the vault has it; otherwise the master password does. */
  async function startExport(): Promise<void> {
    if (hasHello) {
      await exportPasswords(null);
    } else {
      open("export");
    }
  }

  async function submitExport(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const value = formSecret;
    formSecret = "";
    if (!value) return;
    if (await exportPasswords(value)) panel = null;
  }

  async function erase(): Promise<void> {
    panel = null;
    await eraseVault();
  }
</script>

<div class="vault-rows" data-testid="vault-settings">
  {#if helloShown}
    <label class="vault-setting">
      <span class="vault-setting-text">
        <span class="vault-setting-title">{$t("passwords.require_hello_title")}</span>
        <span class="vault-setting-desc">{$t(hasHello ? "passwords.require_hello_description" : "passwords.require_hello_needs")}</span>
      </span>
      <input type="checkbox" role="switch" checked={$vaultStatus?.require_hello === true} disabled={$vaultBusy || !hasHello}
        onchange={toggleRequireHello} data-testid="vault-require-hello" />
    </label>
  {/if}

  <div class="vault-setting">
    <span class="vault-setting-text">
      <span class="vault-setting-title">{$t("passwords.auto_lock")}</span>
    </span>
    <select value={$vaultStatus?.auto_lock_minutes ?? 5} disabled={$vaultBusy} data-testid="vault-auto-lock"
      onchange={event => void chooseAutoLock(Number(event.currentTarget.value))}>
      {#each VAULT_AUTO_LOCK_MINUTES as minutes (minutes)}
        <option value={minutes}>{$t("passwords.minutes", { count: minutes })}</option>
      {/each}
    </select>
  </div>

  <div class="vault-setting">
    <span class="vault-setting-text"><span class="vault-setting-title">{$t("passwords.lock_now")}</span></span>
    <button class="pill-action" type="button" onclick={() => void lockNow()} disabled={$vaultBusy} data-testid="vault-lock">{$t("passwords.lock_now")}</button>
  </div>

  {#if helloShown}
    <div class="vault-setting" data-testid="vault-method-hello-row">
      <span class="vault-setting-text">
        <span class="vault-setting-title">{$t("passwords.unlock_hello_title")}</span>
        <span class="vault-setting-desc">{$t("passwords.unlock_hello_description")}</span>
      </span>
      {#if hasHello}
        <button class="pill-action" type="button" onclick={() => void turnHelloOff()} disabled={$vaultBusy || !hasPassword} data-testid="vault-hello-off">{$t("passwords.hello_turn_off")}</button>
      {:else}
        <button class="pill-action" type="button" onclick={() => void turnHelloOn()} disabled={$vaultBusy} data-testid="vault-hello-on">{$t("passwords.hello_turn_on")}</button>
      {/if}
    </div>
  {/if}

  <div class="vault-setting" data-testid="vault-method-password-row">
    <span class="vault-setting-text">
      <span class="vault-setting-title">{$t("passwords.unlock_password_title")}</span>
      <span class="vault-setting-desc">{$t("passwords.unlock_password_description")}</span>
    </span>
    {#if hasPassword}
      <button class="pill-action" type="button" onclick={() => open("password_off")} disabled={$vaultBusy || !hasHello} data-testid="vault-password-off">{$t("passwords.password_turn_off")}</button>
    {:else}
      <button class="pill-action" type="button" onclick={() => open("password_on")} disabled={$vaultBusy} data-testid="vault-password-on">{$t("passwords.password_turn_on")}</button>
    {/if}
  </div>
  {#if panel === "password_on" || panel === "password_off"}
    <form class="vault-panel" onsubmit={submitPassword} data-testid="vault-password-form">
      <label class="vault-field">
        <span>{$t(panel === "password_on" ? "passwords.new_password" : "passwords.current_password")}</span>
        <input type="password" bind:value={formSecret} autocomplete={panel === "password_on" ? "new-password" : "current-password"}
          minlength={panel === "password_on" ? 8 : undefined} maxlength="256"
          placeholder={panel === "password_on" ? $t("passwords.password_hint") : undefined} disabled={$vaultBusy} required />
      </label>
      <div class="vault-actions">
        <button class="pill-action" type="submit" disabled={$vaultBusy || !formSecret}>{$t(panel === "password_on" ? "passwords.password_confirm_on" : "passwords.password_confirm_off")}</button>
      </div>
    </form>
  {/if}

  {#if hasPassword}
    <div class="vault-setting">
      <span class="vault-setting-text"><span class="vault-setting-title">{$t("passwords.change_password")}</span></span>
      <button class="pill-action" type="button" onclick={() => open("change")} disabled={$vaultBusy} data-testid="vault-change-open">{$t("passwords.change_open")}</button>
    </div>
    {#if panel === "change"}
      <form class="vault-panel" onsubmit={submitChange} data-testid="vault-change-password">
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
      </form>
    {/if}
  {/if}

  <div class="vault-setting">
    <span class="vault-setting-text">
      <span class="vault-setting-title">{$t("passwords.import_title")}</span>
      <span class="vault-setting-desc">{$t("passwords.import_description")}</span>
    </span>
    <button class="pill-action" type="button" onclick={() => void importPasswords()} disabled={$vaultBusy} data-testid="vault-import">{$t("passwords.import_button")}</button>
  </div>

  <div class="vault-setting">
    <span class="vault-setting-text">
      <span class="vault-setting-title">{$t("passwords.export_title")}</span>
      <span class="vault-setting-desc">{$t("passwords.export_description")}</span>
    </span>
    <button class="pill-action" type="button" onclick={() => void startExport()} disabled={$vaultBusy} data-testid="vault-export">{$t("passwords.export_button")}</button>
  </div>
  {#if panel === "export"}
    <form class="vault-panel" onsubmit={submitExport} data-testid="vault-export-form">
      <label class="vault-field">
        <span>{$t("passwords.export_confirm")}</span>
        <input type="password" bind:value={formSecret} autocomplete="current-password" maxlength="256" disabled={$vaultBusy} required />
      </label>
      <div class="vault-actions">
        <button class="pill-action" type="submit" disabled={$vaultBusy || !formSecret}>{$t("passwords.export_button")}</button>
      </div>
    </form>
  {/if}

  <div class="vault-setting">
    <span class="vault-setting-text">
      <span class="vault-setting-title">{$t("passwords.delete_title")}</span>
      <span class="vault-setting-desc">{$t("passwords.delete_description")}</span>
    </span>
    <button class="pill-action" type="button" onclick={() => open("delete")} disabled={$vaultBusy} data-testid="vault-delete-all">{$t("passwords.delete_button")}</button>
  </div>
  {#if panel === "delete"}
    <div role="alertdialog" aria-labelledby="vault-delete-title" class="vault-panel" data-testid="vault-delete-panel">
      <h3 id="vault-delete-title">{$t("passwords.reset_title")}</h3>
      <p class="vault-note">{$t("passwords.reset_warning")}</p>
      <div class="vault-actions">
        <button class="pill-action" type="button" onclick={erase} disabled={$vaultBusy} data-testid="vault-delete-all-confirm">{$t("passwords.reset_confirm")}</button>
        <button class="pill-action" type="button" onclick={() => (panel = null)}>{$t("passwords.cancel")}</button>
      </div>
    </div>
  {/if}
</div>

{#if $vaultInfo}<p class="vault-ok" role="status" data-testid="vault-info">{$t($vaultInfo.key, $vaultInfo.values)}</p>{/if}
{#if $vaultNotice === "hello_on"}<p class="vault-ok" role="status">{$t("passwords.notice_hello_on")}</p>{/if}
{#if $vaultNotice === "hello_off"}<p class="vault-ok" role="status">{$t("passwords.notice_hello_off")}</p>{/if}
{#if $vaultNotice === "password_on"}<p class="vault-ok" role="status">{$t("passwords.notice_password_on")}</p>{/if}
{#if $vaultNotice === "password_off"}<p class="vault-ok" role="status">{$t("passwords.notice_password_off")}</p>{/if}
{#if $vaultNotice === "password_changed"}<p class="vault-ok" role="status">{$t("passwords.password_changed")}</p>{/if}
