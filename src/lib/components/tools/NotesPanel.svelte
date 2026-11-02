<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t } from "../../i18n";
  import { selectedNote, openSavedNote } from "../../stores/toolsContentStore";
  import { currentUrl, tabs, addTab } from "../../stores/browserStore";
  import { noteUrl, noteWriter, notesRevision } from "../../stores/notesStore";
  import { fetchPageNote, searchPageNotes } from "../../services/bridge/notes";
  import type { NoteSummary } from "../../types/browser";
  import { closeTools } from "../../stores/toolsStore";
  let editingUrl = $state(""); let title = $state(""); let loading = $state(true); let error = $state("");
  let query = $state(""); let results = $state<NoteSummary[]>([]); let confirming = $state<string | null>(null);
  let loadRevision = 0;
  $effect(() => noteWriter.retain(editingUrl));
  const draft = $derived.by(() => { void $notesRevision; const value = noteWriter.drafts.get(editingUrl); return value ? { ...value } : undefined; });
  $effect(() => {
    const raw = $selectedNote?.url ?? $currentUrl; const pageTitle = $selectedNote?.title ?? $tabs.find(tab => tab.is_active)?.title ?? raw;
    const revision = ++loadRevision;
    loading = true; error = "";
    void (async () => {
      try {
        const url = noteUrl(raw); const note = noteWriter.drafts.has(url) ? null : await fetchPageNote(url);
        if (revision !== loadRevision) return;
        editingUrl = url; title = pageTitle; noteWriter.seed(url, pageTitle, note?.text ?? "");
        notesRevision.update(value => value + 1);
      } catch (e) { if (revision === loadRevision) { editingUrl = ""; error = describeError(e); } }
      finally { if (revision === loadRevision) loading = false; }
    })();
  });
  $effect(() => {
    const value = query; let disposed = false;
    const timer = setTimeout(() => { void searchPageNotes(value).then(rows => { if (!disposed) results = rows; }).catch(e => { if (!disposed) error = describeError(e); }); }, 180);
    return () => { disposed = true; clearTimeout(timer); };
  });
  async function remove(url: string) { try { await noteWriter.remove(url); results = await searchPageNotes(query); confirming = null; } catch (e) { error = describeError(e); } }
</script>
<h3>{$t("tools.page_notes_adea")}</h3>
{#if $selectedNote}<p data-testid="note-pinned-url">{$selectedNote.url}</p><div class="tool-actions"><button data-testid="note-follow-active" onclick={() => selectedNote.set(null)}>{$t("search.follow_active")}</button><button data-testid="note-open-page" onclick={async () => { const url = $selectedNote?.url; if (url) { await closeTools(); await addTab(url); } }}>{$t("tools.open_page_af03")}</button></div>{/if}
{#if error}<ErrorMessage error={error} />{/if}
{#if loading}<p>{$t("tools.loading_note_7d2d")}</p>
{:else if draft}
  <p>{editingUrl}</p>
  <textarea data-testid="page-note-editor" aria-label={$t("tools.note_for_current_page_0362")} rows="8" value={draft.text} oninput={(event) => noteWriter.edit(editingUrl, title, event.currentTarget.value)}></textarea>
  <p role="status">{$t(({ Saved: "common.saved", Saving: "common.saving", Unsaved: "common.unsaved", Error: "common.error" } as const)[draft.status as "Saved" | "Saving" | "Unsaved" | "Error"])}</p>
  <ErrorMessage error={draft.error} />
  {#if draft.status === "Error"}<button onclick={() => noteWriter.flush(editingUrl)}>{$t("tools.retry_save_71fd")}</button>{/if}
{/if}
<h3>{$t("tools.find_saved_notes_978b")}</h3>
<input aria-label={$t("tools.search_notes_6e7a")} placeholder={$t("tools.search_text_title_or_url_4482")} bind:value={query} />
<button onclick={async () => { try { results = await searchPageNotes(query); } catch (e) { error = describeError(e); } }}>{$t("tools.refresh_saved_notes_87f7")}</button>
<p>{$t("tools.showing_up_to_100_matching_notes_003f")}</p>
{#each results as note (note.url)}
  <article class="tool-card"><strong>{note.title}</strong><span>{note.url}</span><p>{note.preview}</p>
    <div class="tool-actions"><button onclick={() => openSavedNote(note.url).catch(e=>error=describeError(e))}>{$t("search.edit_note")}</button><button onclick={async () => { await closeTools(); await addTab(note.url); }}>{$t("tools.open_page_af03")}</button><button onclick={() => confirming = note.url}>{$t("tools.delete_note_b80e")}</button></div>
    {#if confirming === note.url}<p>{$t("tools.delete_this_note_f806")}</p><div class="tool-actions"><button onclick={() => remove(note.url)}>{$t("tools.confirm_deletion_fa57")}</button><button onclick={() => confirming = null}>{$t("tools.cancel_1976")}</button></div>{/if}
  </article>
{/each}
