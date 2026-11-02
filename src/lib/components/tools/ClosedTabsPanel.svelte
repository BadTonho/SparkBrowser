<script lang="ts">
  import { onMount } from "svelte";
  import { t, formatDate } from "../../i18n";
  import { closedTabs, closedTabsBusy, closedTabsError, closedTabsNotice, refreshClosedTabs, restoreClosedTab, wipeClosedTabs } from "../../stores/closedTabsStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  const desktop = isTauriRuntime(); let confirmClear = $state(false);
  onMount(() => { void refreshClosedTabs(); });
</script>
<h3>{$t("closed.heading")}</h3>
{#if !desktop}<p role="status">{$t("common.desktop_required")}</p>
{:else}
  <p>{$t("closed.description")}</p><ErrorMessage error={$closedTabsError} />
  {#if $closedTabsNotice}<p role="status">{$t($closedTabsNotice === "missing_workspace" ? "closed.missing_workspace" : "closed.empty")}</p>{/if}
  <div class="tool-actions"><button disabled={$closedTabsBusy} onclick={refreshClosedTabs}>{$t("closed.refresh")}</button><button disabled={$closedTabsBusy || !$closedTabs.length} onclick={() => confirmClear = true}>{$t("closed.clear")}</button></div>
  {#if confirmClear}<p>{$t("closed.confirm_clear")}</p><div class="tool-actions"><button disabled={$closedTabsBusy} onclick={async () => { await wipeClosedTabs(); confirmClear = false; }}>{$t("closed.clear")}</button><button onclick={() => confirmClear = false}>{$t("closed.cancel")}</button></div>{/if}
  {#if !$closedTabs.length}<p>{$t("closed.empty")}</p>{/if}
  {#each $closedTabs as row (row.id)}
    <article class="tool-card" data-testid="closed-tab-record"><strong>{row.title}</strong><span>{row.url}</span><span>{row.workspace_name} · {$formatDate(row.closed_at * 1000, { dateStyle: "medium", timeStyle: "short" })}{row.is_pinned ? ` · ${$t("closed.pinned")}` : ""}</span>
      <button disabled={$closedTabsBusy} data-testid="closed-tab-reopen" onclick={() => restoreClosedTab(row.id)}>{$t("closed.reopen")}</button>
    </article>
  {/each}
{/if}
