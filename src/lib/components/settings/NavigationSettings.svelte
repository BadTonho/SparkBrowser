<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import ZoomSettings from "./ZoomSettings.svelte";
  import { t } from "../../i18n";
import SiteSearchSettings from "./SiteSearchSettings.svelte";
import SearchEnginePicker from "./SearchEnginePicker.svelte";
  import { defaultSearchEngine, getSearchEngineLabel } from "../../stores/searchEngineStore";
  import { webSuggestionsEnabled, setWebSuggestionsEnabled } from "../../stores/webSuggestionsStore";
  import { supportsWebSuggestions } from "../../services/address-suggestions.js";
  const webSupported = $derived(supportsWebSuggestions($defaultSearchEngine));
</script>

<section class="setting-group" aria-labelledby="search-heading">
  <h2 class="group-title" id="search-heading"><AppIcon name="search" /> {$t("settings.search_navigation_58d9")}</h2>
  <div class="engine-block" id="setting-default-search-engine" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.default_search_engine_2779")}</span>
      <span class="row-desc">{$t("settings.used_when_typing_search_queries_in_the_omnibox_002d")}</span>
    </div>
    <SearchEnginePicker />
  </div>

  <div class="setting-row" id="setting-web-suggestions" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.web_suggestions")}</span>
      <span class="row-desc">{$t("settings.web_suggestions_description")}</span>
      {#if $webSuggestionsEnabled && !webSupported}
        <span class="row-desc">{$t("settings.web_suggestions_unsupported", { engine: getSearchEngineLabel($defaultSearchEngine) })}</span>
      {/if}
    </div>
    <input
      type="checkbox"
      role="switch"
      aria-label={$t("settings.web_suggestions")}
      checked={$webSuggestionsEnabled}
      onchange={(event) => setWebSuggestionsEnabled(event.currentTarget.checked)}
    />
  </div>
</section>

<SiteSearchSettings />

<ZoomSettings />

<style>
  .engine-block { display: flex; flex-direction: column; gap: 12px; padding: 14px; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); background: rgba(255, 255, 255, 0.03); }
  .engine-block:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 2px; }
</style>
