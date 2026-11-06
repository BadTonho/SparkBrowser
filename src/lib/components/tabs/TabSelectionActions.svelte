<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { t } from "../../i18n";
  import { describeError } from "../../services/user-errors.js";
  import { protectedTab } from "../../services/tab-organizer.js";
  import { closeTabsReviewed } from "../../services/bridge/tab-organizer";
  import { tabs, applyTabsSnapshot } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceStore";
  import { clearTabSelection, selectedTabIds } from "../../stores/tabSelectionStore";

  let reviewing = $state(false);
  let busy = $state(false);
  let message = $state("");
  let closeResult = $state<{ closed: number; preserved: number; closedTitles: string[]; preservedTitles: string[] } | null>(null);
  const selected = $derived($tabs.filter(tab => $selectedTabIds.includes(tab.id) && (!tab.workspace_id || tab.workspace_id === $activeWorkspaceId)));

  async function closeSelected() {
    if (!selected.length || busy) return;
    busy = true;
    message = "";
    const titles = new Map($tabs.map(tab => [tab.id, tab.title]));
    try {
      const result = await closeTabsReviewed(selected.map(tab => tab.id));
      applyTabsSnapshot(result.snapshot);
      closeResult = {
        closed: result.closed.length,
        preserved: result.preserved.length,
        closedTitles: result.closed.map(id => titles.get(id) ?? id),
        preservedTitles: result.preserved.map(item => titles.get(item.id) ?? item.id),
      };
      clearTabSelection();
      reviewing = false;
    } catch (error) {
      message = describeError(error);
    } finally {
      busy = false;
    }
  }
</script>

{#if selected.length}
  <section class="selection-actions" aria-label={$t("tools.tab_organizer_8d2b")}>
    {#if reviewing}
      <strong>{$t("tools.confirm_closing_these_tabs_ae26")}</strong>
      <ul>
        {#each selected as tab (tab.id)}
          <li>{tab.title}{protectedTab(tab) ? ` — ${$t("tools.now_protected_will_be_preserved_5bd4")}` : ""}</li>
        {/each}
      </ul>
      <div class="actions">
        <button class="confirm" disabled={busy} onclick={closeSelected}>{$t("tools.confirm_close_selected_tabs_7bdf")}</button>
        <button disabled={busy} onclick={() => reviewing = false}>{$t("tools.cancel_1976")}</button>
      </div>
    {:else}
      <span>{selected.length} {$t("tools.selected_d7cb")}</span>
      <button class="confirm" disabled={busy} onclick={() => { closeResult = null; message = ""; reviewing = true; }}>{$t("tools.review_aff0")}</button>
      <button disabled={busy} onclick={clearTabSelection}>{$t("tools.clear_selection_cea4")}</button>
    {/if}
    <ErrorMessage error={message} />
  </section>
{/if}

{#if closeResult}
  <section class="selection-result" role="status">
    <p>{$t("common.tabs_closed", closeResult)}</p>
    {#each closeResult.closedTitles as title}<span>{$t("common.closed_label")}: {title}</span>{/each}
    {#each closeResult.preservedTitles as title}<span>{$t("common.preserved_label")}: {title} — {$t("common.protected")}</span>{/each}
  </section>
{/if}

<style>
  .selection-actions,.selection-result { position:fixed; right:18px; bottom:18px; z-index:850; width:min(420px,calc(100vw - 36px)); padding:12px; border:1px solid var(--border-subtle); border-radius:var(--radius-md); background:var(--bg-glass-primary); color:var(--text-primary); box-shadow:0 12px 36px rgba(0,0,0,.38); backdrop-filter:blur(18px); }
  .selection-actions { display:flex; align-items:center; flex-wrap:wrap; gap:8px; }
  .selection-actions strong { flex-basis:100%; }
  .selection-actions ul { flex-basis:100%; max-height:180px; overflow:auto; margin:0; padding-left:20px; }
  .selection-actions li { padding:3px 0; overflow-wrap:anywhere; }
  .actions { display:flex; gap:8px; flex-wrap:wrap; }
  .selection-actions button { padding:6px 9px; border:1px solid var(--border-subtle); border-radius:var(--radius-sm); background:var(--bg-glass-hover); color:var(--text-primary); cursor:pointer; }
  .selection-actions button.confirm { border-color:var(--accent-primary); background:color-mix(in srgb,var(--accent-primary) 22%,transparent); }
  .selection-actions button:disabled { opacity:.55; cursor:wait; }
  .selection-result { display:grid; gap:4px; max-height:32vh; overflow:auto; }
  .selection-result p { margin:0 0 4px; }
  .selection-result span { font-size:12px; overflow-wrap:anywhere; }
</style>
