<script lang="ts">
  import { t } from "../../i18n";
import SiteSearchSettings from "./SiteSearchSettings.svelte";
import {
  defaultSearchEngine,
  SEARCH_ENGINES,
  setDefaultSearchEngine,
  type SearchEngine,
} from "../../stores/settingsStore";
</script>

<section class="setting-group" aria-labelledby="search-heading">
  <h2 class="group-title" id="search-heading">{$t("settings.search_navigation_58d9")}</h2>
  <div class="setting-row" id="setting-default-search-engine" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.default_search_engine_2779")}</span>
      <span class="row-desc">{$t("settings.used_when_typing_search_queries_in_the_omnibox_002d")}</span>
    </div>
    <select
      class="engine-select"
      value={$defaultSearchEngine}
      onchange={(event) => setDefaultSearchEngine(event.currentTarget.value as SearchEngine)}
      aria-label={$t("settings.default_search_engine_f52d")}
    >
      {#each SEARCH_ENGINES as engine (engine.id)}
        <option value={engine.id}>{engine.label}</option>
      {/each}
    </select>
  </div>
</section>

<SiteSearchSettings />
