<script lang="ts">
  import { t } from "../../../i18n";
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
</div>
