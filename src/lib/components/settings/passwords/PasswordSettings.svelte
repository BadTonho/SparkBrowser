<script lang="ts">
  import { onMount } from "svelte";
  import "./passwords.css";
  import { t } from "../../../i18n";
  import { isTauriRuntime } from "../../../services/bridge/runtime";
  import {
    dismissVaultMessages, initializeVaultEvents, refreshVaultStatus, vaultFailure, vaultRecoveryKey, vaultStatus,
  } from "../../../stores/vaultStore";
  import RecoveryKeyDialog from "./RecoveryKeyDialog.svelte";
  import VaultSetup from "./VaultSetup.svelte";
  import VaultUnlock from "./VaultUnlock.svelte";
  import VaultWorkspace from "./VaultWorkspace.svelte";

  const desktop = isTauriRuntime();
  const showUnlock = $derived(desktop && !$vaultRecoveryKey && $vaultStatus?.initialized !== false && $vaultStatus?.unlocked === false);

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

{#snippet vaultMessages()}
  {#if $vaultFailure}
    <p class="vault-error" role="alert" data-testid="vault-error">{$t($vaultFailure.key, $vaultFailure.values)}</p>
  {/if}
  {#if $vaultStatus && $vaultStatus.retry_after_seconds > 0 && !$vaultStatus.unlocked && $vaultFailure?.key !== "passwords.error_throttled"}
    <p class="vault-error" role="status">{$t("passwords.error_throttled", { seconds: $vaultStatus.retry_after_seconds })}</p>
  {/if}
{/snippet}

<section class="setting-group vault-root" aria-labelledby="passwords-heading">
<div class="vault-page" data-testid="vault-page">
  <header class="vault-head">
    <div class="vault-head-text">
      <h2 id="passwords-heading" class="group-title">{$t("passwords.category")}</h2>
      <p class="intro">{$t("passwords.description")}</p>
    </div>
    <aside class="vault-warning" role="note" data-testid="vault-audit-warning">
      <strong>{$t("passwords.warning_title")}</strong>
      <span>{$t("passwords.warning_text")}</span>
    </aside>
  </header>

  {#if !desktop}
    <p class="vault-note">{$t("passwords.desktop_required")}</p>
  {:else if $vaultRecoveryKey}
    <RecoveryKeyDialog />
  {:else if $vaultStatus?.initialized === false}
    <VaultSetup />
  {:else if showUnlock}
    <VaultUnlock messages={vaultMessages} />
  {:else if $vaultStatus?.unlocked}
    <VaultWorkspace />
  {/if}

  {#if !showUnlock}
    {@render vaultMessages()}
  {/if}
</div>
</section>

<style>
  .intro { margin: 4px 0 0; color: var(--text-secondary); font-size: 12px; line-height: 1.6; }
</style>
