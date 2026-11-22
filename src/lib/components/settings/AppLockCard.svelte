<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import {
    addLockWay, loadLockState, lockBusy, lockFailure, lockNotice, lockState, removeLockWay, turnLockOff, turnLockOn,
  } from "../../stores/appLockStore";
  import type { LockMethod, LockProof } from "../../types/appLock";

  // The app lock of the profile in use: a secret asked every time the profile opens. It only blocks
  // access to Spark, and the card says so. Changing an existing lock asks for a way in that works now.
  type Pending =
    | { kind: "password"; secret: string }
    | { kind: "hello" }
    | { kind: "vault"; secret: string }
    | { kind: "remove"; method: LockMethod }
    | { kind: "disable" };

  let firstPassword = $state("");
  let firstVault = $state("");
  let newPassword = $state("");
  let vaultPassword = $state("");
  let pending = $state<Pending | null>(null);
  let proofMethod = $state<LockMethod>("password");
  let proofSecret = $state("");
  let showVaultFirst = $state(false);
  let showVaultAdd = $state(false);

  const lock = $derived($lockState);
  const ways = $derived(lock ? (["password", "hello", "vault"] as const).filter(way => lock[way]) : []);
  const proofWays = $derived(ways.filter(way => way !== "hello" || lock?.hello_available));
  const helloOnly = $derived(!!lock && lock.hello && !lock.password && !lock.vault);

  onMount(() => { void loadLockState(); });

  const wayLabel = (way: LockMethod) => $t(way === "password" ? "lock.way_password" : way === "hello" ? "lock.way_hello" : "lock.way_vault");

  function ask(next: Pending): void {
    pending = next;
    proofSecret = "";
    proofMethod = proofWays[0] ?? "password";
  }

  function stop(): void {
    pending = null;
    proofSecret = "";
  }

  async function confirm(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!pending) return;
    const proof: LockProof = proofMethod === "hello" ? { method: "hello" } : { method: proofMethod, secret: proofSecret };
    let done = false;
    if (pending.kind === "password") done = await addLockWay("password", pending.secret, proof);
    else if (pending.kind === "hello") done = await addLockWay("hello", undefined, proof);
    else if (pending.kind === "vault") done = await addLockWay("vault", pending.secret, proof);
    else if (pending.kind === "remove") done = await removeLockWay(pending.method, proof);
    else done = await turnLockOff(proof);
    proofSecret = "";
    if (done) { pending = null; newPassword = ""; vaultPassword = ""; showVaultAdd = false; }
  }

  async function first(method: LockMethod): Promise<void> {
    const secret = method === "password" ? firstPassword : method === "vault" ? firstVault : undefined;
    if (await turnLockOn(method, secret)) { firstPassword = ""; firstVault = ""; showVaultFirst = false; }
  }
</script>

<div class="setting-row app-lock" id="setting-app-lock" tabindex="-1" data-testid="app-lock-card">
  <div class="row-text">
    <span class="row-label">{$t("lock.settings_title")}</span>
    <span class="row-desc">{$t("lock.settings_description")}</span>
    {#if lock}<span class="row-desc" data-testid="app-lock-status">{$t(lock.enabled ? "lock.settings_on" : "lock.settings_off")}</span>{/if}
    {#if $lockNotice}<span class="row-desc" role="status" data-testid="app-lock-notice">{$t(`lock.notice_${$lockNotice}` as "lock.notice_on")}</span>{/if}
    {#if $lockFailure && !pending}<span class="row-desc" role="alert" data-testid="app-lock-failure">{$t($lockFailure.key, $lockFailure.values)}</span>{/if}

    {#if lock && !lock.enabled}
      <form class="lock-form" onsubmit={event => { event.preventDefault(); void first("password"); }}>
        <label class="row-desc" for="app-lock-first-password">{$t("lock.new_password")}</label>
        <input id="app-lock-first-password" type="password" maxlength="256" autocomplete="new-password" spellcheck="false" placeholder={$t("lock.new_password_placeholder")}
          bind:value={firstPassword} disabled={$lockBusy} data-testid="app-lock-new-password" />
        <div class="lock-buttons">
          <button class="pill-action" type="submit" disabled={$lockBusy || !firstPassword} data-testid="app-lock-enable-password">{$t("lock.turn_on_password")}</button>
          {#if lock.hello_available}
            <button class="pill-action" type="button" disabled={$lockBusy} onclick={() => void first("hello")} data-testid="app-lock-enable-hello">{$t("lock.add_hello")}</button>
          {/if}
          {#if lock.vault_available}
            <button class="pill-action" type="button" disabled={$lockBusy} onclick={() => (showVaultFirst = !showVaultFirst)} data-testid="app-lock-enable-vault">{$t("lock.add_vault")}</button>
          {/if}
        </div>
      </form>
      {#if showVaultFirst}
        <form class="lock-form" onsubmit={event => { event.preventDefault(); void first("vault"); }}>
          <label class="row-desc" for="app-lock-first-vault">{$t("lock.vault_password")}</label>
          <input id="app-lock-first-vault" type="password" maxlength="256" autocomplete="off" spellcheck="false" bind:value={firstVault} disabled={$lockBusy} data-testid="app-lock-vault-input" />
          <button class="pill-action" type="submit" disabled={$lockBusy || !firstVault} data-testid="app-lock-vault-submit">{$t("lock.add_vault")}</button>
        </form>
      {/if}
    {:else if lock}
      <ul class="lock-ways" data-testid="app-lock-ways">
        {#each ["password", "hello", "vault"] as const as way (way)}
          <li data-testid={`app-lock-way-${way}`} data-on={lock[way]}>
            <span>{wayLabel(way)}: {$t(lock[way] ? "lock.way_on" : "lock.way_off")}</span>
            {#if lock[way] && ways.length > 1}
              <button class="pill-action" type="button" disabled={$lockBusy} onclick={() => ask({ kind: "remove", method: way })} data-testid={`app-lock-remove-${way}`}>{$t("lock.remove")}</button>
            {/if}
          </li>
        {/each}
      </ul>
      {#if helloOnly}<span class="row-desc" data-testid="app-lock-hello-warning">{$t("lock.hello_only_warning")}</span>{/if}

      <form class="lock-form" onsubmit={event => { event.preventDefault(); if (newPassword) ask({ kind: "password", secret: newPassword }); }}>
        <label class="row-desc" for="app-lock-change-password">{$t(lock.password ? "lock.change_password" : "lock.set_password")}</label>
        <input id="app-lock-change-password" type="password" maxlength="256" autocomplete="new-password" spellcheck="false" placeholder={$t("lock.new_password_placeholder")}
          bind:value={newPassword} disabled={$lockBusy} data-testid="app-lock-change-input" />
        <button class="pill-action" type="submit" disabled={$lockBusy || !newPassword} data-testid="app-lock-change-password-submit">{$t(lock.password ? "lock.change_password" : "lock.set_password")}</button>
      </form>

      <div class="lock-buttons">
        {#if !lock.hello && lock.hello_available}
          <button class="pill-action" type="button" disabled={$lockBusy} onclick={() => ask({ kind: "hello" })} data-testid="app-lock-add-hello">{$t("lock.add_hello")}</button>
        {/if}
        {#if !lock.vault && lock.vault_available}
          <button class="pill-action" type="button" disabled={$lockBusy} onclick={() => (showVaultAdd = !showVaultAdd)} data-testid="app-lock-add-vault">{$t("lock.add_vault")}</button>
        {/if}
        <button class="pill-action" type="button" disabled={$lockBusy} onclick={() => ask({ kind: "disable" })} data-testid="app-lock-disable">{$t("lock.turn_off_lock")}</button>
      </div>
      {#if showVaultAdd}
        <form class="lock-form" onsubmit={event => { event.preventDefault(); if (vaultPassword) ask({ kind: "vault", secret: vaultPassword }); }}>
          <label class="row-desc" for="app-lock-add-vault-input">{$t("lock.vault_password")}</label>
          <input id="app-lock-add-vault-input" type="password" maxlength="256" autocomplete="off" spellcheck="false" bind:value={vaultPassword} disabled={$lockBusy} data-testid="app-lock-add-vault-input" />
          <button class="pill-action" type="submit" disabled={$lockBusy || !vaultPassword} data-testid="app-lock-add-vault-submit">{$t("lock.add_vault")}</button>
        </form>
      {/if}

      {#if pending}
        <form class="lock-form lock-confirm" onsubmit={confirm} data-testid="app-lock-confirm-panel">
          <span class="row-label">{$t("lock.confirm_title")}</span>
          {#if proofWays.length > 1}
            <label class="row-desc" for="app-lock-proof-method">{$t("lock.confirm_with")}</label>
            <select id="app-lock-proof-method" bind:value={proofMethod} disabled={$lockBusy} data-testid="app-lock-confirm-method">
              {#each proofWays as way (way)}<option value={way}>{wayLabel(way)}</option>{/each}
            </select>
          {/if}
          {#if proofMethod === "hello"}
            <span class="row-desc">{$t("lock.confirm_hello_hint")}</span>
          {:else}
            <label class="row-desc" for="app-lock-proof-secret">{$t("lock.confirm_secret")}</label>
            <input id="app-lock-proof-secret" type="password" maxlength="256" autocomplete="off" spellcheck="false" bind:value={proofSecret} disabled={$lockBusy} data-testid="app-lock-confirm-secret" />
          {/if}
          {#if $lockFailure}<span class="row-desc" role="alert" data-testid="app-lock-confirm-error">{$t($lockFailure.key, $lockFailure.values)}</span>{/if}
          <div class="lock-buttons">
            <button class="pill-action" type="submit" disabled={$lockBusy || (proofMethod !== "hello" && !proofSecret)} data-testid="app-lock-confirm-submit">{$t("lock.confirm_submit")}</button>
            <button class="pill-action" type="button" onclick={stop} data-testid="app-lock-confirm-cancel">{$t("lock.confirm_cancel")}</button>
          </div>
        </form>
      {/if}
    {/if}
  </div>
</div>

<style>
  input[type="password"], select { min-width: 0; width: min(320px, 100%); padding: 7px 9px; border-radius: 7px; border: 1px solid var(--border-subtle); background: var(--bg-glass-elevated); color: var(--text-primary); font-size: 12px; font-family: inherit; }
  input[type="password"]:disabled { opacity: 0.6; }
  .row-text { display: flex; flex-direction: column; gap: 6px; align-items: flex-start; }
  .lock-form { display: flex; flex-direction: column; gap: 6px; align-items: flex-start; margin-top: 6px; }
  .lock-buttons { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 4px; }
  .lock-ways { display: flex; flex-direction: column; gap: 6px; margin: 4px 0 0; padding: 0; list-style: none; }
  .lock-ways li { display: flex; align-items: center; gap: 10px; font-size: 12.5px; }
  .lock-confirm { padding: 10px 12px; border: 1px solid var(--border-subtle); border-radius: 10px; }
</style>
