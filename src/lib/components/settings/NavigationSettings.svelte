<script lang="ts">
import {
  defaultSearchEngine,
  SEARCH_ENGINES,
  setDefaultSearchEngine,
  type SearchEngine,
} from "../../stores/settingsStore";
</script>

<section class="setting-group" aria-labelledby="search-heading">
  <h2 class="group-title" id="search-heading">🔎 Search & Navigation</h2>
  <div class="setting-row" id="setting-default-search-engine" tabindex="-1">
    <div class="row-text">
      <span class="row-label">Default Search Engine</span>
      <span class="row-desc">Used when typing search queries in the Omnibox</span>
    </div>
    <select
      class="engine-select"
      value={$defaultSearchEngine}
      onchange={(event) => setDefaultSearchEngine(event.currentTarget.value as SearchEngine)}
      aria-label="Default search engine"
    >
      {#each SEARCH_ENGINES as engine (engine.id)}
        <option value={engine.id}>{engine.label}</option>
      {/each}
    </select>
  </div>
</section>
