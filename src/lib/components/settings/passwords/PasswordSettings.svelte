<script lang="ts">
  import { onMount } from "svelte";
  import "./passwords.css";
  import { t } from "../../../i18n";
  import { isTauriRuntime } from "../../../services/bridge/runtime";
  import {
    dismissVaultMessages, initializeVaultEvents, refreshVaultStatus, vaultFailure, vaultRecoveryKey, vaultStatus,
  } from "../../../stores/vaultStore";
  import RecoveryKeyDialog from "./RecoveryKeyDialog.svelte";
  import VaultList from "./VaultList.svelte";
  import VaultOptions from "./VaultOptions.svelte";
  import VaultSetup from "./VaultSetup.svelte";
  import VaultUnlock from "./VaultUnlock.svelte";

  const desktop = isTauriRuntime();

  onMount(() => {
    dismissVaultMessages();
    void refreshVaultStatus();
    const stop = initializeVaultEvents();
    return () => {
      stop();
      dismissVaultMessages();
    };
  });
</script>

<section class="setting-group" aria-labelledby="passwords-heading">
  <h2 id="passwords-heading" class="group-title">{$t("passwords.category")}</h2>
  <p class="intro">{$t("passwords.description")}</p>
  <aside class="vault-warning" role="note" data-testid="vault-audit-warning">
    <strong>{$t("passwords.warning_title")}</strong>
    <span>{$t("passwords.warning_text")}</span>
  </aside>

  {#if !desktop}
    <p class="vault-note">{$t("passwords.desktop_required")}</p>
  {:else if $vaultRecoveryKey}
    <RecoveryKeyDialog />
  {:else if $vaultStatus?.initialized === false}
    <VaultSetup />
  {:else if $vaultStatus?.unlocked === false}
    <VaultUnlock />
  {:else if $vaultStatus?.unlocked}
    <VaultList />
    <VaultOptions />
  {/if}

  {#if $vaultFailure}
    <p class="vault-error" role="alert" data-testid="vault-error">{$t($vaultFailure.key, $vaultFailure.values)}</p>
  {/if}
  {#if $vaultStatus && $vaultStatus.retry_after_seconds > 0 && !$vaultStatus.unlocked && !$vaultFailure}
    <p class="vault-error" role="status">{$t("passwords.error_throttled", { seconds: $vaultStatus.retry_after_seconds })}</p>
  {/if}
</section>

<style>
  .intro { margin: 4px 0 0; color: var(--text-secondary); font-size: 12px; line-height: 1.6; }
</style>
