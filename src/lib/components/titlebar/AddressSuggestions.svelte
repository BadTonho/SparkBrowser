<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { suggestionState, chooseSuggestion } from "../../stores/addressSuggestionsStore";
  import { defaultSearchEngine, getSearchEngineLabel } from "../../stores/searchEngineStore";
  import type { Suggestion } from "../../services/address-suggestions.js";

  const ROW_HEIGHT = 46;
  let left = $state(0);
  let width = $state(0);
  const engine = $derived(getSearchEngineLabel($defaultSearchEngine));
  const visible = $derived($suggestionState.open && $suggestionState.items.length > 0);

  // The list lines up with the address bar it belongs to; measured when it opens and when the window resizes.
  function measure() {
    const box = document.querySelector(".omnibox-wrapper");
    if (!box) return;
    const rect = box.getBoundingClientRect();
    left = Math.round(rect.left);
    width = Math.round(rect.width);
  }
  $effect(() => { if (visible) measure(); });
  onMount(() => {
    window.addEventListener("resize", measure);
    return () => window.removeEventListener("resize", measure);
  });

  function label(item: Suggestion): string {
    if (item.kind === "search") return $t("titlebar.suggest_search", { engine });
    if (item.kind === "navigate") return $t("titlebar.suggest_go");
    if (item.kind === "bookmark") return $t("titlebar.suggest_bookmark");
    if (item.kind === "web") return $t("titlebar.suggest_web");
    return $t("titlebar.suggest_history");
  }
</script>

{#if visible}
  <div class="address-suggestions">
    <div
      id="address-suggestions-list"
      class="suggestion-panel"
      role="listbox"
      aria-label={$t("titlebar.suggestions_label")}
      style:margin-left="{left}px"
      style:width="{width}px"
      style:min-height="{$suggestionState.rows * ROW_HEIGHT + 12}px"
    >
      {#each $suggestionState.items as item, index (item.id)}
        <!-- mousedown is cancelled so the address bar keeps focus while a row is clicked. -->
        <button
          type="button"
          id="address-suggestion-{index}"
          class="suggestion"
          class:highlighted={index === $suggestionState.selected}
          role="option"
          aria-selected={index === $suggestionState.selected}
          onmousedown={(event) => event.preventDefault()}
          onclick={() => void chooseSuggestion(item)}
        >
          <span class="suggestion-icon" aria-hidden="true">
            {#if item.kind === "bookmark"}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"><path d="m12 3 2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9Z" /></svg>
            {:else if item.kind === "history"}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg>
            {:else if item.kind === "navigate"}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9" /><path d="M3 12h18M12 3c3 3.5 3 14.5 0 18M12 3c-3 3.5-3 14.5 0 18" /></svg>
            {:else}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" /></svg>
            {/if}
          </span>
          <span class="suggestion-text">
            <span class="suggestion-primary">{item.primary}</span>
            {#if item.kind === "search" || item.kind === "navigate"}
              <span class="suggestion-secondary">{label(item)}</span>
            {:else if item.secondary}
              <span class="suggestion-secondary">{item.secondary}</span>
            {/if}
          </span>
          {#if item.kind === "bookmark" || item.kind === "history" || item.kind === "web"}
            <span class="suggestion-kind">{label(item)}</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .address-suggestions { flex: 0 0 auto; min-width: 0; padding: 0 0 8px; }
  .suggestion-panel {
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px;
    border: var(--modal-surface-border, 1px solid var(--border-subtle));
    border-radius: 16px;
    background: var(--modal-surface-bg, var(--bg-glass-elevated));
    backdrop-filter: var(--modal-surface-filter, none);
    box-shadow: var(--modal-surface-shadow, 0 12px 32px rgba(0, 0, 0, 0.4));
  }
  .suggestion {
    display: flex;
    align-items: center;
    gap: 12px;
    box-sizing: border-box;
    width: 100%;
    height: 44px;
    padding: 0 12px;
    border-radius: 11px;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .suggestion:hover, .suggestion.highlighted { background: rgba(255, 255, 255, 0.08); }
  .suggestion.highlighted { background: rgba(255, 255, 255, 0.12); }
  .suggestion-icon { flex: 0 0 auto; display: grid; place-items: center; width: 24px; color: var(--text-muted); }
  .suggestion-text { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .suggestion-primary, .suggestion-secondary { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  .suggestion-primary { font-size: 13px; font-weight: 600; }
  .suggestion-secondary { color: var(--text-muted); font-size: 11.5px; }
  .suggestion-kind { flex: 0 0 auto; padding: 2px 8px; border-radius: 999px; background: rgba(255, 255, 255, 0.07); color: var(--text-secondary); font-size: 10.5px; font-weight: 600; }
  :global(html[data-theme="light"]) .suggestion:hover { background: rgba(0, 0, 0, 0.05); }
  :global(html[data-theme="light"]) .suggestion.highlighted { background: rgba(0, 0, 0, 0.09); }
  :global(html[data-theme="light"]) .suggestion-kind { background: rgba(0, 0, 0, 0.06); }
</style>
