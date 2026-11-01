<script lang="ts">
  import { tabs, applyTabsSnapshot, selectTab } from "../../stores/browserStore";
  import { activeWorkspaceId, workspaces } from "../../stores/workspaceStore";
  import { closeTools } from "../../stores/toolsStore";
  import { protectedTab, duplicateGroups, duplicateCandidates, tabUrlKey } from "../../services/tab-organizer.js";
  import { closeTabsReviewed } from "../../services/bridge/tab-organizer";
  let allWorkspaces = $state(false); let query = $state(""); let domain = $state(""); let onlyDuplicates = $state(false);
  let selected = $state<string[]>([]); let reviewing = $state(false); let busy = $state(false); let message = $state("");
  const scoped = $derived($tabs.filter(tab => allWorkspaces || tab.workspace_id === $activeWorkspaceId));
  function host(raw: string) { try { return new URL(raw).hostname; } catch { return ""; } }
  const domains = $derived([...new Set(scoped.map(tab => host(tab.url)))].sort());
  const groups = $derived(duplicateGroups(scoped));
  const duplicateKeys = $derived(new Set(groups.map(([key]) => key)));
  const visible = $derived(scoped.filter(tab => (!domain || host(tab.url) === domain) && (!query || `${tab.title} ${tab.url}`.toLowerCase().includes(query.toLowerCase())) && (!onlyDuplicates || duplicateKeys.has(tabUrlKey(tab.url)))));
  const chosen = $derived($tabs.filter(tab => selected.includes(tab.id)));
  async function closeSelected() {
    busy = true; message = "";
    try { const result = await closeTabsReviewed(selected); applyTabsSnapshot(result.snapshot); message = `${result.closed.length} closed; ${result.preserved.length} preserved. ${result.preserved.map(item => `${item.id}: ${item.reason}`).join(" · ")}`; selected = []; reviewing = false; }
    catch (error) { message = String(error); } finally { busy = false; }
  }
</script>
<h3>Tab organizer</h3>
<div class="tool-actions">
  <label><input type="checkbox" bind:checked={allWorkspaces} /> All workspaces</label>
  <label><input type="checkbox" bind:checked={onlyDuplicates} /> Duplicates only</label>
</div>
<input aria-label="Search open tabs" placeholder="Search title or URL" bind:value={query} />
<select aria-label="Filter tabs by domain" bind:value={domain}><option value="">All domains</option>{#each domains as item}<option value={item}>{item}</option>{/each}</select>
<div class="tool-actions">
  <button disabled={busy} onclick={() => { selected = duplicateCandidates(visible); reviewing = false; }}>Select extra duplicates</button>
  <button disabled={busy || !selected.length} onclick={() => reviewing = true}>Review {selected.length} selected</button>
  <button disabled={busy} onclick={() => { selected = []; reviewing = false; }}>Clear selection</button>
</div>
<p>{groups.length} duplicate groups. Active, pinned, loading, edited-form, media and downloading tabs are protected.</p>
{#if message}<p role="status">{message}</p>{/if}
{#if reviewing}
  <article class="tool-card"><strong>Confirm closing these tabs</strong>
    {#each chosen as tab (tab.id)}<span>{tab.title}{protectedTab(tab) ? " — now protected; will be preserved" : ""}</span>{/each}
    <div class="tool-actions"><button disabled={busy} onclick={closeSelected}>Confirm close selected tabs</button><button disabled={busy} onclick={() => reviewing = false}>Cancel</button></div>
  </article>
{/if}
{#each visible as tab (tab.id)}
  <article class="tool-card">
    <label><input type="checkbox" disabled={busy || protectedTab(tab)} checked={selected.includes(tab.id)} onchange={(event) => { selected = event.currentTarget.checked ? [...selected, tab.id] : selected.filter(id => id !== tab.id); reviewing = false; }} /> {tab.title}</label>
    <span>{tab.url}</span><span>{$workspaces.find(ws => ws.id === tab.workspace_id)?.name ?? tab.workspace_id}{protectedTab(tab) ? " · Protected" : ""}{duplicateKeys.has(tabUrlKey(tab.url)) ? " · Duplicate" : ""}</span>
    <button onclick={async () => { await closeTools(); await selectTab(tab.id); }}>Go to tab</button>
  </article>
{/each}
