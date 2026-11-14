<script lang="ts">
  import { t, type MessageKey } from "../../i18n";
  import { defaultSearchEngine, SEARCH_ENGINES, setDefaultSearchEngine } from "../../stores/settingsStore";
  import SearchEngineIcon from "../icons/SearchEngineIcon.svelte";
</script>

<!--
  One tile per engine; the chosen one is outlined. Plain buttons (not a <select>) so the choice is visible at a
  glance and also works in the native settings surface, which only forwards clicks on buttons. Colors live in
  the icon's fill attribute because inline styles are not copied into that surface.
-->
<div class="picker" role="radiogroup" aria-label={$t("settings.default_search_engine_f52d")} data-testid="search-engine-picker">
  {#each SEARCH_ENGINES as engine (engine.id)}
    <button
      type="button"
      class="tile"
      class:selected={$defaultSearchEngine === engine.id}
      role="radio"
      aria-checked={$defaultSearchEngine === engine.id}
      data-testid={"search-engine-" + engine.id}
      onclick={() => setDefaultSearchEngine(engine.id)}
    >
      <span class="logo" aria-hidden="true"><SearchEngineIcon id={engine.id} color={engine.color} /></span>
      <span class="text">
        <strong>{engine.label}</strong>
        <small>{$t(`settings.engine_${engine.id}_desc` as MessageKey)}</small>
      </span>
    </button>
  {/each}
</div>

<style>
  .picker { display: grid; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 10px; }
  /* Every tile reserves the same 2px border, so selecting one never shifts the layout. */
  .tile { display: flex; min-width: 0; align-items: center; gap: 12px; padding: 11px 12px; border: 2px solid transparent; border-radius: 12px; background: rgba(255, 255, 255, 0.04); box-shadow: inset 0 0 0 1px var(--border-subtle); color: var(--text-primary); text-align: left; transition: border-color 0.16s ease, background-color 0.16s ease; }
  .tile:hover { background: rgba(255, 255, 255, 0.07); }
  .tile:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 2px; }
  .tile.selected { border-color: var(--accent-primary); box-shadow: none; }
  /* A light tile keeps dark brand colors (Yahoo, Kagi) readable on the dark theme. */
  .logo { display: grid; flex: 0 0 auto; width: 38px; height: 38px; place-items: center; border-radius: 10px; background: #fff; box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25); }
  .text { display: flex; min-width: 0; flex: 1; flex-direction: column; gap: 2px; }
  strong { overflow: hidden; font-size: 13px; font-weight: 700; line-height: 1.3; text-overflow: ellipsis; white-space: nowrap; }
  small { color: var(--text-secondary); font-size: 11.5px; line-height: 1.4; overflow-wrap: anywhere; }
  @media (prefers-reduced-motion: reduce) { .tile { transition: none; } }
</style>
