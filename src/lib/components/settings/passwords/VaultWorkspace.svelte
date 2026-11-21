<script lang="ts">
  import { t } from "../../../i18n";
  import { addTestPasswords, removeTestPasswords, vaultBusy, vaultInfo } from "../../../stores/vaultStore";
  import VaultList from "./VaultList.svelte";
  import VaultSettings from "./VaultSettings.svelte";

  // Two pages, like the password manager of other browsers: the passwords and the vault's own settings.
  let tab = $state<"passwords" | "settings">("passwords");
</script>

<div class="vault-workspace" data-testid="vault-workspace">
  <div class="vault-tabs" role="tablist" aria-label={$t("passwords.category")}>
    <button type="button" role="tab" class="vault-tab" aria-selected={tab === "passwords"} onclick={() => (tab = "passwords")} data-testid="vault-tab-passwords">{$t("passwords.tab_passwords")}</button>
    <button type="button" role="tab" class="vault-tab" aria-selected={tab === "settings"} onclick={() => (tab = "settings")} data-testid="vault-tab-settings">{$t("passwords.tab_settings")}</button>
  </div>
  {#if tab === "passwords"}
    <VaultList />
  {:else}
    <VaultSettings />
  {/if}
  {#if import.meta.env.DEV}
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
