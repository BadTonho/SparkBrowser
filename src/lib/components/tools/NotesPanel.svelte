<script lang="ts">
  import { currentUrl, tabs, addTab } from "../../stores/browserStore";
  import { noteUrl, noteWriter, notesRevision } from "../../stores/notesStore";
  import { fetchPageNote, searchPageNotes } from "../../services/bridge/notes";
  import type { PageNote } from "../../types/browser";
  import { closeTools } from "../../stores/toolsStore";
  let editingUrl = $state(""); let title = $state(""); let loading = $state(true); let error = $state("");
  let query = $state(""); let results = $state<PageNote[]>([]); let confirming = $state<string | null>(null);
  let loadRevision = 0;
  const draft = $derived.by(() => { void $notesRevision; const value = noteWriter.drafts.get(editingUrl); return value ? { ...value } : undefined; });
  $effect(() => {
    const raw = $currentUrl; const pageTitle = $tabs.find(tab => tab.is_active)?.title ?? raw;
    const revision = ++loadRevision;
    loading = true; error = "";
    void (async () => {
      try {
        const url = noteUrl(raw); const note = await fetchPageNote(url);
        if (revision !== loadRevision) return;
        editingUrl = url; title = pageTitle; noteWriter.seed(url, pageTitle, note?.text ?? "");
        notesRevision.update(value => value + 1);
      } catch (e) { if (revision === loadRevision) { editingUrl = ""; error = String(e); } }
      finally { if (revision === loadRevision) loading = false; }
    })();
  });
  $effect(() => {
    const value = query; let disposed = false;
    const timer = setTimeout(() => { void searchPageNotes(value).then(rows => { if (!disposed) results = rows; }).catch(e => { if (!disposed) error = String(e); }); }, 180);
    return () => { disposed = true; clearTimeout(timer); };
  });
  async function remove(url: string) { try { await noteWriter.remove(url); results = await searchPageNotes(query); confirming = null; } catch (e) { error = String(e); } }
</script>
<h3>Page notes</h3>
{#if error}<p role="alert">{error}</p>{/if}
{#if loading}<p>Loading note…</p>
{:else if draft}
  <p>{editingUrl}</p>
  <textarea aria-label="Note for current page" rows="8" value={draft.text} oninput={(event) => noteWriter.edit(editingUrl, title, event.currentTarget.value)}></textarea>
  <p role="status">{draft.status}{draft.error ? `: ${draft.error}` : ""}</p>
  {#if draft.status === "Error"}<button onclick={() => noteWriter.flush(editingUrl)}>Retry save</button>{/if}
{/if}
<h3>Find saved notes</h3>
<input aria-label="Search notes" placeholder="Search text, title or URL" bind:value={query} />
<button onclick={async () => { try { results = await searchPageNotes(query); } catch (e) { error = String(e); } }}>Refresh saved notes</button>
<p>Showing up to 100 matching notes.</p>
{#each results as note (note.url)}
  <article class="tool-card"><strong>{note.title}</strong><span>{note.url}</span><p>{note.text.slice(0, 180)}</p>
    <div class="tool-actions"><button onclick={async () => { await closeTools(); await addTab(note.url); }}>Open page</button><button onclick={() => confirming = note.url}>Delete note</button></div>
    {#if confirming === note.url}<p>Delete this note?</p><div class="tool-actions"><button onclick={() => remove(note.url)}>Confirm deletion</button><button onclick={() => confirming = null}>Cancel</button></div>{/if}
  </article>
{/each}
