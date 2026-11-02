<script lang="ts">
  import { onMount } from "svelte";
  import { t, formatNumber } from "../../i18n";
  import { fetchDiagnostics, type NativeDiagnostics } from "../../services/bridge/diagnostics";
  import { invoke } from "../../services/bridge/runtime";
  import { describeError } from "../../services/user-errors.js";
  import { applyTabsSnapshot, selectTab } from "../../stores/browserStore";
  import { activeWorkspaceId, workspaces } from "../../stores/workspaceStore";
  import { closeTools } from "../../stores/toolsStore";
  import { normalizeSearch } from "../../services/command-ranking.js";
  import type { TabSnapshot } from "../../types/browser";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  let diagnostics = $state<NativeDiagnostics | null>(null);
  let all = $state(false); let query = $state(""); let error = $state(""); let busy = $state(false);
  let revision = 0; let disposed = false;
  const scoped = $derived(diagnostics?.tab_details.filter(t => all || t.workspace_id === $activeWorkspaceId) ?? []);
  const visible = $derived(scoped.filter(t => normalizeSearch(`${t.title} ${t.url}`).includes(normalizeSearch(query))));
  async function refresh() {
    const request = ++revision;
    try { const result = await fetchDiagnostics(); if (!disposed && request === revision) diagnostics = result; }
    catch (e) { if (!disposed && request === revision) error = describeError(e); }
  }
  async function hibernate(id: string) {
    busy = true; error = "";
    try { applyTabsSnapshot(await invoke<TabSnapshot>("hibernate_tab", { tabId: id })); await refresh(); }
    catch (e) { error = describeError(e); await refresh(); }
    finally { busy = false; }
  }
  onMount(() => { void refresh(); const timer = setInterval(() => { void refresh(); }, 2000); return () => { disposed = true; ++revision; clearInterval(timer); }; });
</script>
<h3>{$t("diagnostics.heading")}</h3>
<p>{$t("diagnostics.description")}</p>
<label><input data-testid="diagnostics-all" type="checkbox" bind:checked={all} /> {$t("tools.all_workspaces_415c")}</label>
<input data-testid="diagnostics-query" aria-label={$t("tools.search_open_tabs_e50f")} placeholder={$t("tools.search_title_or_url_78b6")} bind:value={query} />
<ErrorMessage error={error} />
{#if diagnostics}
  <p data-testid="diagnostics-totals">{$t("diagnostics.totals", { resident: scoped.filter(t=>t.resident).length, hibernated: scoped.filter(t=>!t.resident).length, protected: scoped.filter(t=>t.protection_reasons.length).length, limit: diagnostics.resident_limit })}</p>
  {#each visible as tab (tab.id)}
    <article class="tool-card" data-testid="diagnostic-tab" data-tab-id={tab.id}>
      <strong>{tab.title}</strong><span>{tab.url}</span>
      <span>{$workspaces.find(w=>w.id===tab.workspace_id)?.name ?? tab.workspace_id} · {$t(tab.resident ? "diagnostics.resident" : "diagnostics.hibernated")}</span>
      <span>{$t("diagnostics.last_active", { seconds: $formatNumber(tab.seconds_since_activation) })}</span>
      <span data-testid="diagnostic-reasons">{tab.protection_reasons.length ? tab.protection_reasons.map(reason=>$t(`diagnostics.${reason}`)).join(", ") : $t("diagnostics.unprotected")}</span>
      <div class="tool-actions">
        <button onclick={async () => { await closeTools(); await selectTab(tab.id); }}>{$t("tools.go_to_tab_060a")}</button>
        <button data-testid="diagnostic-hibernate" disabled={busy || !tab.resident || !!tab.protection_reasons.length} onclick={() => hibernate(tab.id)}>{$t("diagnostics.hibernate")}</button>
      </div>
    </article>
  {/each}
{:else}<p>{$t("common.desktop_required")}</p>{/if}
