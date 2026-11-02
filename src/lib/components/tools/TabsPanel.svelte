<script lang="ts">
import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t } from "../../i18n";
  import SplitControls from "./SplitControls.svelte";
  import { tabs, applyTabsSnapshot, selectTab } from "../../stores/browserStore";
  import { activeWorkspaceId, workspaces } from "../../stores/workspaceStore";
  import { closeTools } from "../../stores/toolsStore";
  import { protectedTab, duplicateGroups, duplicateCandidates, tabUrlKey } from "../../services/tab-organizer.js";
  import { closeTabsReviewed } from "../../services/bridge/tab-organizer";
  let allWorkspaces = $state(false); let query = $state(""); let domain = $state(""); let onlyDuplicates = $state(false);
  let selected = $state<string[]>([]); let reviewing = $state(false); let busy = $state(false); let message = $state("");
  let closeResult = $state<{ closed: number; preserved: number; closedTitles: string[]; preservedTitles: string[] } | null>(null);
  const scoped = $derived($tabs.filter(tab => allWorkspaces || tab.workspace_id === $activeWorkspaceId));
  function host(raw: string) { try { return new URL(raw).hostname; } catch { return ""; } }
  const domains = $derived([...new Set(scoped.map(tab => host(tab.url)))].sort());
  const groups = $derived(duplicateGroups(scoped));
  const duplicateKeys = $derived(new Set(groups.map(([key]) => key)));
  const visible = $derived(scoped.filter(tab => (!domain || host(tab.url) === domain) && (!query || `${tab.title} ${tab.url}`.toLowerCase().includes(query.toLowerCase())) && (!onlyDuplicates || duplicateKeys.has(tabUrlKey(tab.url)))));
  const chosen = $derived($tabs.filter(tab => selected.includes(tab.id)));
  async function closeSelected() {
    busy = true; message = "";
    const titles = new Map($tabs.map(tab => [tab.id, tab.title]));
    try { const result = await closeTabsReviewed(selected); applyTabsSnapshot(result.snapshot); closeResult = { closed: result.closed.length, preserved: result.preserved.length, closedTitles: result.closed.map(id => titles.get(id) ?? id), preservedTitles: result.preserved.map(item => titles.get(item.id) ?? item.id) }; selected = []; reviewing = false; }
    catch (error) { message = describeError(error); } finally { busy = false; }
  }
</script>
<SplitControls />
<h3>{$t("tools.tab_organizer_8d2b")}</h3>
<div class="tool-actions">
  <label><input type="checkbox" bind:checked={allWorkspaces} /> {$t("tools.all_workspaces_415c")}</label>
  <label><input type="checkbox" bind:checked={onlyDuplicates} /> {$t("tools.duplicates_only_fd7e")}</label>
</div>
<input aria-label={$t("tools.search_open_tabs_e50f")} placeholder={$t("tools.search_title_or_url_78b6")} bind:value={query} />
<select aria-label={$t("tools.filter_tabs_by_domain_4e89")} bind:value={domain}><option value="">{$t("tools.all_domains_0f97")}</option>{#each domains as item}<option value={item}>{item}</option>{/each}</select>
<div class="tool-actions">
  <button disabled={busy} onclick={() => { selected = duplicateCandidates(visible); reviewing = false; }}>{$t("tools.select_extra_duplicates_f631")}</button>
  <button disabled={busy || !selected.length} onclick={() => reviewing = true}>{$t("tools.review_aff0")} {selected.length} {$t("tools.selected_d7cb")}</button>
  <button disabled={busy} onclick={() => { selected = []; reviewing = false; }}>{$t("tools.clear_selection_cea4")}</button>
</div>
<p>{groups.length} {$t("tools.duplicate_groups_active_pinned_loading_edited_form_media_and_downloading_44e9")}</p>
<ErrorMessage error={message} />
{#if closeResult}
  <p role="status">{$t("common.tabs_closed", closeResult)}</p>
  <ul>{#each closeResult.closedTitles as title}<li>{$t("common.closed_label")}: {title}</li>{/each}
    {#each closeResult.preservedTitles as title}<li>{$t("common.preserved_label")}: {title} — {$t("common.protected")}</li>{/each}</ul>
{/if}
{#if reviewing}
  <article class="tool-card"><strong>{$t("tools.confirm_closing_these_tabs_ae26")}</strong>
    {#each chosen as tab (tab.id)}<span>{tab.title}{protectedTab(tab) ? $t("tools.now_protected_will_be_preserved_5bd4") : ""}</span>{/each}
    <div class="tool-actions"><button disabled={busy} onclick={closeSelected}>{$t("tools.confirm_close_selected_tabs_7bdf")}</button><button disabled={busy} onclick={() => reviewing = false}>{$t("tools.cancel_1976")}</button></div>
  </article>
{/if}
{#each visible as tab (tab.id)}
  <article class="tool-card">
    <label><input type="checkbox" disabled={busy || protectedTab(tab)} checked={selected.includes(tab.id)} onchange={(event) => { selected = event.currentTarget.checked ? [...selected, tab.id] : selected.filter(id => id !== tab.id); reviewing = false; }} /> {tab.title}</label>
    <span>{tab.url}</span><span>{$workspaces.find(ws => ws.id === tab.workspace_id)?.name ?? tab.workspace_id}{protectedTab(tab) ? $t("tools.protected_8022") : ""}{duplicateKeys.has(tabUrlKey(tab.url)) ? $t("tools.duplicate_68b1") : ""}</span>
    <button onclick={async () => { await closeTools(); await selectTab(tab.id); }}>{$t("tools.go_to_tab_060a")}</button>
  </article>
{/each}
