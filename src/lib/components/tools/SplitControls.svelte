<script lang="ts">
  import { tabs, applyTabsSnapshot } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { splitLayout } from "../../stores/splitViewStore";
  import { setSplitView } from "../../services/bridge/split-view";
  let left = $state(""); let right = $state(""); let busy = $state(false); let error = $state("");
  const candidates = $derived($tabs.filter(tab => tab.workspace_id === $activeWorkspaceId));
  async function set(enabled: boolean) { busy = true; error = ""; try { applyTabsSnapshot(await setSplitView(enabled ? left : null, enabled ? right : null)); } catch (e) { error = String(e); } finally { busy = false; } }
</script>
<article class="tool-card">
  <strong>Split view</strong><p>Choose two tabs from this workspace. Click a page to focus its navigation controls.</p>
  <div class="tool-actions">
    <select aria-label="Left split tab" bind:value={left}><option value="">Left page</option>{#each candidates as tab (tab.id)}<option value={tab.id}>{tab.title}</option>{/each}</select>
    <select aria-label="Right split tab" bind:value={right}><option value="">Right page</option>{#each candidates as tab (tab.id)}<option value={tab.id}>{tab.title}</option>{/each}</select>
    <button disabled={busy || !left || !right || left === right} onclick={() => set(true)}>Split selected tabs</button>
    {#if $splitLayout}<button disabled={busy} onclick={() => set(false)}>Exit split view</button>{/if}
  </div>
  {#if error}<p role="alert">{error}</p>{/if}
</article>
