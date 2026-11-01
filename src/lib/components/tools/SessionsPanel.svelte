<script lang="ts">
  import { onMount } from "svelte";
  import type { SavedSession } from "../../types/browser";
  import { fetchSavedSessions, saveNamedSession, renameNamedSession, deleteNamedSession, restoreNamedSession } from "../../services/bridge/saved-sessions";
  import { applyTabsSnapshot } from "../../stores/browserStore";
  let sessions = $state<SavedSession[]>([]);
  let name = $state("");
  let busy = $state(false);
  let message = $state("");
  let confirming = $state<number | null>(null);
  let renameId = $state<number | null>(null);
  let renameName = $state("");
  async function run(action: () => Promise<void>) {
    if (busy) return;
    busy = true; message = "";
    try { await action(); sessions = await fetchSavedSessions(); }
    catch (error) { message = String(error); }
    finally { busy = false; }
  }
  onMount(() => { void run(async () => {}); });
  async function restore(id: number) {
    const result = await restoreNamedSession(id);
    applyTabsSnapshot(result.snapshot);
    message = result.warnings.join(" · ") || "Session added to the current workspace.";
  }
</script>

<h3>Named sessions</h3>
<p>Save the current workspace's pages. Restoring adds tabs without closing your existing pages.</p>
<form onsubmit={(event) => { event.preventDefault(); void run(async () => { await saveNamedSession(name); name = ""; }); }}>
  <input aria-label="Session name" placeholder="Session name" bind:value={name} maxlength="100" required />
  <button disabled={busy || !name.trim()}>Save workspace</button>
</form>
{#if message}<p role="status">{message}</p>{/if}
{#if !sessions.length}<p>No saved sessions yet.</p>{/if}
{#each sessions as session (session.id)}
  <article class="tool-card">
    <strong>{session.name}</strong><span>{session.pages.length} pages</span>
    <div class="tool-actions">
      <button disabled={busy} onclick={() => run(() => restore(session.id))}>Restore</button>
      <button disabled={busy} onclick={() => { renameId = session.id; renameName = session.name; }}>Rename</button>
      <button disabled={busy} onclick={() => { confirming = confirming === session.id ? null : session.id; }}>Update / Delete</button>
    </div>
    {#if renameId === session.id}
      <form onsubmit={(event) => { event.preventDefault(); void run(async () => { await renameNamedSession(session.id, renameName); renameId = null; }); }}>
        <input aria-label="Rename session" bind:value={renameName} maxlength="100" required />
        <button disabled={busy}>Save name</button><button type="button" onclick={() => renameId = null}>Cancel</button>
      </form>
    {/if}
    {#if confirming === session.id}
      <p>Replace the saved pages with this workspace, or delete this saved session?</p>
      <div class="tool-actions">
        <button disabled={busy} onclick={() => run(async () => { await saveNamedSession(session.name, session.id); confirming = null; })}>Confirm update</button>
        <button disabled={busy} onclick={() => run(async () => { await deleteNamedSession(session.id); confirming = null; })}>Confirm deletion</button>
        <button onclick={() => confirming = null}>Cancel</button>
      </div>
    {/if}
  </article>
{/each}
