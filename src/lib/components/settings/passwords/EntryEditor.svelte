<script lang="ts">
  import { t } from "../../../i18n";
  import { generatePassword } from "../../../services/vault/password-generator.js";
  import { saveEntry, scanTotpQr, vaultBusy } from "../../../stores/vaultStore";
  import type { VaultEntryData } from "../../../types/vault";
  import TotpCode from "./TotpCode.svelte";

  let { id, initial, onclose }: { id: string | null; initial: VaultEntryData; onclose: () => void } = $props();

  // The editor starts from what it is given; a new entry is opened with {#key} instead of being updated.
  /* svelte-ignore state_referenced_locally */
  let title = $state(initial.title);
  /* svelte-ignore state_referenced_locally */
  let url = $state(initial.url);
  /* svelte-ignore state_referenced_locally */
  let username = $state(initial.username);
  /* svelte-ignore state_referenced_locally */
  let password = $state(initial.password);
  /* svelte-ignore state_referenced_locally */
  let notes = $state(initial.notes);
  /* svelte-ignore state_referenced_locally */
  let totp = $state(initial.totp);
  let visible = $state(false);
  let totpVisible = $state(false);

  let scanned = $state(false);

  /** Picks a picture of the QR code: the backend opens the file dialog, reads it and only accepts a valid 2FA setup. */
  async function scan(): Promise<void> {
    scanned = false;
    const secret = await scanTotpQr();
    if (secret) {
      totp = secret;
      scanned = true;
    }
  }

  function generate(): void {
    password = generatePassword();
    visible = true;
  }

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (await saveEntry(id, { title, url, username, password, notes, totp: totp.trim() })) {
      password = "";
      notes = "";
      totp = "";
      onclose();
    }
  }

  function cancel(): void {
    password = "";
    notes = "";
    totp = "";
    onclose();
  }
</script>

<form class="vault-card" onsubmit={submit} data-testid="vault-editor">
  <h3>{$t(id ? "passwords.edit_title" : "passwords.add_title")}</h3>
  <label class="vault-field">
    <span>{$t("passwords.field_title")}</span>
    <input bind:value={title} maxlength="200" required autocomplete="off" disabled={$vaultBusy} data-testid="vault-field-title" />
  </label>
  <label class="vault-field">
    <span>{$t("passwords.field_website")}</span>
    <input bind:value={url} maxlength="2048" autocomplete="off" inputmode="url" disabled={$vaultBusy} data-testid="vault-field-url" />
  </label>
  <label class="vault-field">
    <span>{$t("passwords.field_username")}</span>
    <input bind:value={username} maxlength="320" autocomplete="off" disabled={$vaultBusy} data-testid="vault-field-username" />
  </label>
  <div class="vault-field">
    <label for="vault-field-password">{$t("passwords.field_password")}</label>
    <div class="vault-row">
      <input id="vault-field-password" type={visible ? "text" : "password"} bind:value={password} maxlength="1024"
        autocomplete="new-password" spellcheck="false" disabled={$vaultBusy} data-testid="vault-field-password" />
      <button class="pill-action" type="button" onclick={() => (visible = !visible)}>{$t(visible ? "passwords.hide" : "passwords.show")}</button>
      <button class="pill-action" type="button" onclick={generate} data-testid="vault-generate">{$t("passwords.generate")}</button>
    </div>
  </div>
  <div class="vault-field">
    <label for="vault-field-totp">{$t("passwords.field_totp")}</label>
    <div class="vault-row">
      <input id="vault-field-totp" type={totpVisible ? "text" : "password"} bind:value={totp} maxlength="1024" placeholder={$t("passwords.totp_placeholder")}
        autocomplete="off" spellcheck="false" disabled={$vaultBusy} data-testid="vault-field-totp" />
      <button class="pill-action" type="button" onclick={() => (totpVisible = !totpVisible)}>{$t(totpVisible ? "passwords.hide" : "passwords.show")}</button>
    </div>
    <div class="vault-row">
      <button class="pill-action" type="button" disabled={$vaultBusy} onclick={scan} data-testid="vault-totp-qr">{$t("passwords.totp_scan")}</button>
    </div>
    {#if scanned}<p class="vault-ok" role="status" data-testid="vault-totp-scanned">{$t("passwords.totp_scan_done")}</p>{/if}
    <TotpCode secret={totp} preview />
    <p class="vault-totp-advice">{$t("passwords.totp_advice")}</p>
  </div>
  <label class="vault-field">
    <span>{$t("passwords.field_notes")}</span>
    <textarea bind:value={notes} maxlength="5000" disabled={$vaultBusy} data-testid="vault-field-notes"></textarea>
  </label>
  <div class="vault-actions">
    <button class="pill-action primary-pill" type="submit" disabled={$vaultBusy || !title.trim()} data-testid="vault-save">{$t("passwords.save")}</button>
    <button class="pill-action" type="button" onclick={cancel}>{$t("passwords.cancel")}</button>
  </div>
</form>
