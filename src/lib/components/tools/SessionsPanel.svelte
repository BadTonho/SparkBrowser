<script lang="ts">
import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { openSavedSession } from "../../stores/toolsContentStore";
  import { t, plural } from "../../i18n";
  import { onMount } from "svelte";
  import type { SavedSession } from "../../types/browser";
  import { fetchSavedSessions, saveNamedSession, renameNamedSession, deleteNamedSession, restoreNamedSession } from "../../services/bridge/saved-sessions";
  import { applyTabsSnapshot } from "../../stores/browserStore";
  let sessions = $state<SavedSession[]>([]);
  let name = $state("");
  let busy = $state(false);
  let message = $state("");
  let restored = $state(false);
  let warnings = $state<string[]>([]);
  let confirming = $state<number | null>(null);
  let renameId = $state<number | null>(null);
  let renameName = $state("");
  async function run(action: () => Promise<void>) {
    if (busy) return;
    busy = true; message = "";
    try { await action(); sessions = await fetchSavedSessions(); }
    catch (error) { message = describeError(error); }
    finally { busy = false; }
  }
  onMount(() => { void run(async () => {}); });
  async function restore(id: number) {
    const result = await restoreNamedSession(id);
    applyTabsSnapshot(result.snapshot);
    restored = true; warnings = result.warnings;
  }
</script>

<h3>{$t("tools.named_sessions_f542")}</h3>
<p>{$t("tools.save_the_current_workspace_s_pages_restoring_adds_tabs_without_closing_y_7344")}</p>
<form onsubmit={(event) => { event.preventDefault(); void run(async () => { await saveNamedSession(name); name = ""; }); }}>
  <input aria-label={$t("tools.session_name_136a")} placeholder={$t("tools.session_name_136a")} bind:value={name} maxlength="100" required />
  <button disabled={busy || !name.trim()}>{$t("tools.save_workspace_6a9c")}</button>
</form>
<ErrorMessage error={message} />
{#if restored}<p role="status">{$t(warnings.length ? "common.session_warning" : "common.session_added")}</p>{#if warnings.length}<details><summary>{$t("common.technical_details")}</summary><pre>{warnings.join("\n")}</pre></details>{/if}{/if}
{#if !sessions.length}<p>{$t("tools.no_saved_sessions_yet_baa3")}</p>{/if}
{#each sessions as session (session.id)}
  <article class="tool-card">
    <strong>{session.name}</strong><span>{$plural(session.pages.length, $t("common.pages_one"), $t("common.pages_other"))}</span>
    <div class="tool-actions">
      <button disabled={busy} onclick={() => run(() => openSavedSession(session.id))}>{$t("search.details")}</button>
      <button disabled={busy} onclick={() => run(() => restore(session.id))}>{$t("tools.restore_a76e")}</button>
      <button disabled={busy} onclick={() => { renameId = session.id; renameName = session.name; }}>{$t("tools.rename_3064")}</button>
      <button disabled={busy} onclick={() => { confirming = confirming === session.id ? null : session.id; }}>{$t("tools.update_delete_bf40")}</button>
    </div>
    {#if renameId === session.id}
      <form onsubmit={(event) => { event.preventDefault(); void run(async () => { await renameNamedSession(session.id, renameName); renameId = null; }); }}>
        <input aria-label={$t("tools.rename_session_2cad")} bind:value={renameName} maxlength="100" required />
        <button disabled={busy}>{$t("tools.save_name_b729")}</button><button type="button" onclick={() => renameId = null}>{$t("tools.cancel_1976")}</button>
      </form>
    {/if}
    {#if confirming === session.id}
      <p>{$t("tools.replace_the_saved_pages_with_this_workspace_or_delete_this_saved_session_71f8")}</p>
      <div class="tool-actions">
        <button disabled={busy} onclick={() => run(async () => { await saveNamedSession(session.name, session.id); confirming = null; })}>{$t("tools.confirm_update_913d")}</button>
        <button disabled={busy} onclick={() => run(async () => { await deleteNamedSession(session.id); confirming = null; })}>{$t("tools.confirm_deletion_fa57")}</button>
        <button onclick={() => confirming = null}>{$t("tools.cancel_1976")}</button>
      </div>
    {/if}
  </article>
{/each}
