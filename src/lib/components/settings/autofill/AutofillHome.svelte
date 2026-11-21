<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../../i18n";
  import { autofillSettings, changeAutofillSettings, loadAutofillSettings } from "../../../stores/autofillStore";
  import { isPrivateWindow } from "../../../stores/privateWindowStore";
  import { vaultBusy } from "../../../stores/vaultStore";
  import type { AutofillPage } from "../../../types/autofill";
  import VaultIcon from "../passwords/VaultIcon.svelte";

  let { onopen }: { onopen: (page: Exclude<AutofillPage, "home">) => void } = $props();

  const rows = [
    { page: "passwords", label: "autofill.passwords", hint: "autofill.passwords_hint", testid: "autofill-open-passwords" },
    { page: "payments", label: "autofill.payments", hint: "autofill.payments_hint", testid: "autofill-open-payments" },
    { page: "contacts", label: "autofill.contacts", hint: "autofill.contacts_hint", testid: "autofill-open-contacts" },
  ] as const;

  onMount(() => { void loadAutofillSettings(); });

  function toggleEnabled(event: Event): void {
    const settings = $autofillSettings;
    if (settings) void changeAutofillSettings({ ...settings, enabled: (event.currentTarget as HTMLInputElement).checked });
  }
  function togglePrivate(event: Event): void {
    const settings = $autofillSettings;
    if (settings) void changeAutofillSettings({ ...settings, in_private: (event.currentTarget as HTMLInputElement).checked });
  }
</script>

<ul class="vault-groups autofill-home" data-testid="autofill-home">
  {#each rows as row (row.page)}
    <li class="vault-group">
      <button class="vault-group-head" type="button" onclick={() => onopen(row.page)} data-testid={row.testid}>
        <span class="vault-group-label">
          <span class="vault-group-name">{$t(row.label)}</span>
          <span class="vault-group-count">{$t(row.hint)}</span>
        </span>
        <span class="vault-chevron"><VaultIcon name="chevron" /></span>
      </button>
    </li>
  {/each}
</ul>

{#if !$isPrivateWindow}
  <div class="vault-rows autofill-switches">
    <label class="vault-setting">
      <span class="vault-setting-text">
        <span class="vault-setting-title">{$t("autofill.enabled_title")}</span>
        <span class="vault-setting-desc">{$t("autofill.enabled_description")}</span>
      </span>
      <input type="checkbox" role="switch" checked={$autofillSettings?.enabled ?? true} disabled={$vaultBusy || !$autofillSettings}
        onchange={toggleEnabled} data-testid="autofill-enabled" />
    </label>
    <label class="vault-setting">
      <span class="vault-setting-text">
        <span class="vault-setting-title">{$t("autofill.private_title")}</span>
        <span class="vault-setting-desc">{$t("autofill.private_description")}</span>
      </span>
      <input type="checkbox" role="switch" checked={$autofillSettings?.in_private ?? false} disabled={$vaultBusy || !$autofillSettings || !$autofillSettings.enabled}
        onchange={togglePrivate} data-testid="autofill-private" />
    </label>
  </div>
{/if}

<style>
  .autofill-switches { margin-top: 18px; }
</style>
