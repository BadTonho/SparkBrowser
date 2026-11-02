<script lang="ts">
  import { onMount } from "svelte";
  import { t, plural } from "../../i18n";
  import { selectedSession } from "../../stores/toolsContentStore";
  import { fetchSavedSession } from "../../services/bridge/command-search";
  import { restoreNamedSession } from "../../services/bridge/saved-sessions";
  import { describeError, UserError } from "../../services/user-errors.js";
  import { applyTabsSnapshot } from "../../stores/browserStore";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  let busy = $state(true); let error = $state(""); let restored = $state(false); let warnings = $state<string[]>([]);
  onMount(() => {
    let disposed = false; const id = $selectedSession?.id;
    if (id !== undefined) void fetchSavedSession(id).then(s => { if (!disposed && $selectedSession?.id === id) selectedSession.set(s); }).catch(e => { if (!disposed) error = describeError(e); }).finally(() => { if (!disposed) busy = false; });
    return () => { disposed = true; };
  });
  async function restore() {
    const session = $selectedSession; if (!session || busy) return;
    busy = true; error = "";
    try {
      const current = await fetchSavedSession(session.id);
      if (JSON.stringify(current) !== JSON.stringify(session)) {
        selectedSession.set(current);
        throw new UserError("command_search.changed", "Review the updated session before restoring");
      }
      const result = await restoreNamedSession(session.id, current);
      applyTabsSnapshot(result.snapshot); warnings = result.warnings; restored = true;
    } catch (e) { error = describeError(e); }
    finally { busy = false; }
  }
</script>
{#if $selectedSession}
  <h3 data-testid="session-details">{$selectedSession.name}</h3>
  <p>{$plural($selectedSession.pages.length, $t("common.pages_one"), $t("common.pages_other"))}</p>
  <p>{$t("tools.save_the_current_workspace_s_pages_restoring_adds_tabs_without_closing_y_7344")}</p>
  <ErrorMessage error={error} />
  {#if restored}<p role="status">{$t(warnings.length ? "common.session_warning" : "common.session_added")}</p>{/if}
  {#if warnings.length}<details><summary>{$t("common.technical_details")}</summary><pre>{warnings.join("\n")}</pre></details>{/if}
  {#each $selectedSession.pages as page, index}
    <article class="tool-card"><strong>{index+1}. {page.title}</strong><span>{page.url}</span>{#if page.is_pinned}<span>{$t("search.pinned")}</span>{/if}</article>
  {/each}
  <div class="tool-actions"><button data-testid="session-details-restore" disabled={busy} onclick={restore}>{$t("tools.restore_a76e")}</button><button onclick={() => selectedSession.set(null)}>{$t("search.back_sessions")}</button></div>
{/if}
