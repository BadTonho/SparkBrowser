<script lang="ts">
  import { onMount } from "svelte";
  import type { SiteSearchShortcut } from "../../types/browser";
  import { fetchSiteSearchShortcuts, saveSiteSearchShortcut, deleteSiteSearchShortcut } from "../../services/bridge/site-search";
  let items = $state<SiteSearchShortcut[]>([]);
  let keyword = $state(""); let name = $state(""); let template = $state(""); let previous = $state<string | null>(null);
  let error = $state(""); let busy = $state(false);
  async function run(action: () => Promise<void>) { if (busy) return; busy = true; error = ""; try { await action(); items = await fetchSiteSearchShortcuts(); } catch (e) { error = String(e); } finally { busy = false; } }
  function reset() { keyword = ""; name = ""; template = ""; previous = null; }
  onMount(() => { void run(async () => {}); });
</script>
<section class="setting-group site-search" aria-labelledby="site-search-heading">
  <h2 class="group-title" id="site-search-heading">Site search shortcuts</h2>
  <p>Type a keyword followed by your query in the address bar or Command Bar. No shortcuts are created automatically.</p>
  <form onsubmit={(event) => { event.preventDefault(); void run(async () => { await saveSiteSearchShortcut({ keyword, name, template }, previous); reset(); }); }}>
    <input aria-label="Search shortcut name" placeholder="Name" bind:value={name} maxlength="100" required />
    <input aria-label="Search keyword" placeholder="Keyword" bind:value={keyword} maxlength="24" required />
    <input aria-label="Search URL template" placeholder={"https://example.com/search?q={query}"} bind:value={template} required />
    <button disabled={busy}>{previous ? "Save changes" : "Add shortcut"}</button>
    {#if previous}<button type="button" onclick={reset}>Cancel</button>{/if}
  </form>
  {#if error}<p role="alert">{error}</p>{/if}
  {#each items as item (item.keyword)}
    <div class="setting-row"><span>{item.keyword} · {item.name}</span>
      <button disabled={busy} onclick={() => { previous = item.keyword; keyword = item.keyword; name = item.name; template = item.template; }}>Edit</button>
      <button disabled={busy} onclick={() => run(async () => { await deleteSiteSearchShortcut(item.keyword); if (previous === item.keyword) reset(); })}>Remove</button>
    </div>
  {/each}
</section>
<style>
  .site-search form { display: grid; gap: 10px; }
  .site-search input { width: 100%; padding: 10px; color: var(--text-primary); background: #ffffff08; border: 1px solid var(--border-subtle); border-radius: 6px; }
  .site-search button { color: var(--text-primary); background: #ffffff10; padding: 8px 12px; border-radius: 6px; cursor: pointer; }
</style>
