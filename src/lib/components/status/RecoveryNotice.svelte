<script lang="ts">
  import { t, plural } from "../../i18n";
  import { recoveryStatus, recoveryError, recoveryDismissed, dismissRecovery } from "../../stores/recoveryStore";
  import { openTools } from "../../stores/toolsStore";
  import ErrorMessage from "./ErrorMessage.svelte";
</script>
{#if !$recoveryDismissed && ($recoveryStatus?.recovered || $recoveryError)}
  <aside data-testid="recovery-notice" role="status">
    <ErrorMessage error={$recoveryError} />
    {#if $recoveryStatus?.recovered}
      <span>{$plural($recoveryStatus.restored_tabs, $t("recovery.one"), $t("recovery.other"))}</span>
      <button data-testid="recovery-organizer" onclick={() => openTools("tabs")}>{$t("recovery.organize")}</button>
    {/if}
    <button data-testid="recovery-dismiss" onclick={dismissRecovery}>{$t("recovery.dismiss")}</button>
  </aside>
{/if}
<style>aside { display: flex; gap: 12px; align-items: center; padding: 6px 14px; font-size: 12px; background: var(--bg-glass-elevated); } button { color: var(--accent-primary); }</style>
