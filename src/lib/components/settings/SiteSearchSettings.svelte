<script lang="ts">
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t } from "../../i18n";
  import { onMount } from "svelte";
  import type { SiteSearchShortcut } from "../../types/browser";
  import { fetchSiteSearchShortcuts, saveSiteSearchShortcut, deleteSiteSearchShortcut } from "../../services/bridge/site-search";
  let items = $state<SiteSearchShortcut[]>([]);
  let keyword = $state(""); let name = $state(""); let template = $state(""); let previous = $state<string | null>(null);
  let error = $state(""); let busy = $state(false);
  async function run(action: () => Promise<void>) { if (busy) return; busy = true; error = ""; try { await action(); items = await fetchSiteSearchShortcuts(); } catch (e) { error = describeError(e); } finally { busy = false; } }
  function reset() { keyword = ""; name = ""; template = ""; previous = null; }
  onMount(() => { void run(async () => {}); });
</script>
<section class="setting-group site-search" aria-labelledby="site-search-heading">
  <h2 class="group-title" id="site-search-heading">{$t("settings.site_search_shortcuts_b00d")}</h2>
  <p>{$t("settings.type_a_keyword_followed_by_your_query_in_the_address_bar_or_command_bar__3148")}</p>
  <form onsubmit={(event) => { event.preventDefault(); void run(async () => { await saveSiteSearchShortcut({ keyword, name, template }, previous); reset(); }); }}>
    <input aria-label={$t("settings.search_shortcut_name_7392")} placeholder={$t("settings.name_dcd1")} bind:value={name} maxlength="100" required />
    <input aria-label={$t("settings.search_keyword_45ed")} placeholder={$t("settings.keyword_a668")} bind:value={keyword} maxlength="24" required />
    <input aria-label={$t("settings.search_url_template_4385")} placeholder={"https://example.com/search?q={query}"} bind:value={template} required />
    <button disabled={busy}>{previous ? $t("settings.save_changes_dd0a") : $t("settings.add_shortcut_2393")}</button>
    {#if previous}<button type="button" onclick={reset}>{$t("settings.cancel_1976")}</button>{/if}
  </form>
  {#if error}<ErrorMessage error={error} />{/if}
  {#each items as item (item.keyword)}
    <div class="setting-row"><span>{item.keyword} · {item.name}</span>
      <button disabled={busy} onclick={() => { previous = item.keyword; keyword = item.keyword; name = item.name; template = item.template; }}>{$t("settings.edit_464c")}</button>
      <button disabled={busy} onclick={() => run(async () => { await deleteSiteSearchShortcut(item.keyword); if (previous === item.keyword) reset(); })}>{$t("settings.remove_c381")}</button>
    </div>
  {/each}
</section>
<style>
  .site-search form { display: grid; gap: 10px; }
  .site-search input { width: 100%; padding: 10px; color: var(--text-primary); background: #ffffff08; border: 1px solid var(--border-subtle); border-radius: 6px; }
  .site-search button { color: var(--text-primary); background: #ffffff10; padding: 8px 12px; border-radius: 6px; cursor: pointer; }
</style>
