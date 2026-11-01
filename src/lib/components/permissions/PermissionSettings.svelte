<script lang="ts">
  import { onMount } from "svelte";
  import type { SitePermission, PermissionSnapshot } from "../../types/browser";
  import { fetchSitePermissions, setSitePermission } from "../../services/bridge/site-permissions";
  let { origin = null }: { origin?: string | null } = $props();
  let snapshot = $state<PermissionSnapshot | null>(null); let error = $state(""); let busy = $state(false);
  const kinds: SitePermission["kind"][] = ["camera", "microphone", "location", "notifications"];
  async function refresh() { try { snapshot = await fetchSitePermissions(); } catch (e) { error = String(e); } }
  async function change(site: string, kind: SitePermission["kind"], state: SitePermission["state"]) { busy = true; error = ""; try { await setSitePermission(site, kind, state); await refresh(); } catch (e) { error = String(e); } finally { busy = false; } }
  onMount(() => { void refresh(); });
</script>
<section class="permission-settings" aria-labelledby="site-permissions-heading">
  <h2 id="site-permissions-heading">Site permissions</h2>
  {#if error}<p role="alert">{error}</p>{/if}
  {#if !snapshot}<p>Loading native permissions…</p>
  {:else if !snapshot.supported}<p role="status">{snapshot.message}</p>
  {:else}
    <p>Decisions are shared across workspaces. Changes apply to new requests. Active capture may continue until the page is reloaded.</p>
    <button disabled={busy} onclick={refresh}>Refresh decisions</button>
    {#if origin}
      <p>{origin}</p>
      {#each kinds as kind}
        <label>{kind}<select aria-label={`${kind} permission`} disabled={busy} value={snapshot.settings.find(row => row.origin === origin && row.kind === kind)?.state ?? "ask"} onchange={(event) => change(origin!, kind, event.currentTarget.value as SitePermission["state"])}><option value="ask">Ask</option><option value="allow">Allow</option><option value="block">Block</option></select></label>
      {/each}
    {:else}
      {#if !snapshot.settings.length}<p>No saved site decisions.</p>{/if}
      {#each snapshot.settings as row (`${row.origin}:${row.kind}`)}
        <label>{row.origin} · {row.kind}<select aria-label={`${row.origin} ${row.kind} permission`} disabled={busy} value={row.state} onchange={(event) => change(row.origin, row.kind, event.currentTarget.value as SitePermission["state"])}><option value="ask">Ask</option><option value="allow">Allow</option><option value="block">Block</option></select></label>
      {/each}
    {/if}
  {/if}
</section>
<style>
  .permission-settings { display: flex; flex-direction: column; gap: 14px; }
  .permission-settings label { display: flex; justify-content: space-between; align-items: center; gap: 16px; overflow-wrap: anywhere; }
  .permission-settings select, .permission-settings button { padding: 8px; background: #ffffff10; color: var(--text-primary, #eee); border: 1px solid var(--border-subtle, #555); border-radius: 6px; }
</style>
