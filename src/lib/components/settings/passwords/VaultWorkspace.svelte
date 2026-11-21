<script lang="ts">
  import { t } from "../../../i18n";
  import { addTestPasswords, removeTestPasswords, vaultBusy, vaultInfo } from "../../../stores/vaultStore";
  import type { AutofillPage } from "../../../types/autofill";
  import AutofillHome from "../autofill/AutofillHome.svelte";
  import RecordList from "../autofill/RecordList.svelte";
  import VaultIcon from "./VaultIcon.svelte";
  import VaultList from "./VaultList.svelte";
  import VaultSettings from "./VaultSettings.svelte";

  // Two pages, like the password manager of other browsers: the passwords and the vault's own settings.
  let tab = $state<"passwords" | "settings">("passwords");
  // The section opens on a list (password manager, payments, contact details), like other browsers.
  let page = $state<AutofillPage>("home");
</script>

<div class="vault-workspace" data-testid="vault-workspace">
  {#if page === "home"}
    <AutofillHome onopen={next => (page = next)} />
  {:else if page === "payments"}
    <RecordList kind="card" onback={() => (page = "home")} />
  {:else if page === "contacts"}
    <RecordList kind="contact" onback={() => (page = "home")} />
  {:else}
  <div class="vault-detail-head">
    <button class="vault-icon-button" type="button" onclick={() => (page = "home")} aria-label={$t("passwords.detail_back")} data-testid="autofill-back"><VaultIcon name="back" /></button>
    <h3>{$t("autofill.passwords")}</h3>
  </div>
  <div class="vault-tabs" role="tablist" aria-label={$t("passwords.category")}>
    <button type="button" role="tab" class="vault-tab" aria-selected={tab === "passwords"} onclick={() => (tab = "passwords")} data-testid="vault-tab-passwords">{$t("passwords.tab_passwords")}</button>
    <button type="button" role="tab" class="vault-tab" aria-selected={tab === "settings"} onclick={() => (tab = "settings")} data-testid="vault-tab-settings">{$t("passwords.tab_settings")}</button>
  </div>
  {#if tab === "passwords"}
    <VaultList />
  {:else}
    <VaultSettings />
  {/if}
  {/if}
  {#if import.meta.env.DEV && page === "passwords"}
    <!-- Only in development builds (npm run tauri dev): made-up logins to try the list with data. -->
    <aside class="vault-dev" data-testid="vault-dev-tools">
      <strong>{$t("passwords.dev_title")}</strong>
      <span>{$t("passwords.dev_note")}</span>
      <div class="vault-actions">
        <button class="pill-action" type="button" disabled={$vaultBusy} onclick={() => void addTestPasswords()} data-testid="vault-dev-seed">{$t("passwords.dev_add")}</button>
        <button class="pill-action" type="button" disabled={$vaultBusy} onclick={() => void removeTestPasswords()} data-testid="vault-dev-remove">{$t("passwords.dev_remove")}</button>
      </div>
      {#if $vaultInfo && ($vaultInfo.key === "passwords.dev_added" || $vaultInfo.key === "passwords.dev_removed")}
        <span class="vault-ok" role="status">{$t($vaultInfo.key, $vaultInfo.values)}</span>
      {/if}
    </aside>
  {/if}
</div>
