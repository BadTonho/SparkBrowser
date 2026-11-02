<script lang="ts">
  import { t } from "../../i18n";
  import { closedTabsError, closedTabsNotice, dismissClosedNotice } from "../../stores/closedTabsStore";
  import { isToolsOpen } from "../../stores/toolsStore";
  import ErrorMessage from "./ErrorMessage.svelte";
</script>
{#if !$isToolsOpen && ($closedTabsNotice || $closedTabsError)}
  <aside role="status" data-testid="closed-tabs-notice"><ErrorMessage error={$closedTabsError} />
    {#if $closedTabsNotice}<span>{$t($closedTabsNotice === "missing_workspace" ? "closed.missing_workspace" : "closed.empty")}</span>{/if}
    <button onclick={dismissClosedNotice} aria-label={$t("closed.dismiss")}>×</button>
  </aside>
{/if}
<style>aside { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 6px 14px; font-size: 12px; background: var(--bg-glass-elevated); }</style>
