<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { suggestionState, suggestionsSession, beginSuggestionsSession, endSuggestionsSession, closeSuggestions, chooseSuggestion } from "../../stores/addressSuggestionsStore";
  import { defaultSearchEngine, getSearchEngineLabel } from "../../stores/searchEngineStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { mountPanelHost } from "../panels/panel-host";
  import { onSuggestionsPopoverAction, setSuggestionsPopover } from "../../services/bridge/suggestions-popover";
  import type { Suggestion } from "../../services/address-suggestions.js";
  import "./suggestions.css";

  const ROW_HEIGHT = 46;
  const native = isTauriRuntime();
  let left = $state(0);
  let width = $state(0);
  const engine = $derived(getSearchEngineLabel($defaultSearchEngine));
  const visible = $derived($suggestionState.open && $suggestionState.items.length > 0);
  // In the app the list is drawn by a native popover: the session starts when there is something to list and ends when not.
  const present = $derived(native ? Boolean($suggestionsSession) : visible);

  // In the page layout (outside the app) the list lines up with the address bar it belongs to.
  function measure() {
    const box = document.querySelector(".omnibox-wrapper");
    if (!box) return;
    const rect = box.getBoundingClientRect();
    left = Math.round(rect.left);
    width = Math.round(rect.width);
  }
  $effect(() => {
    if (!native) { if (visible) measure(); return; }
    if (visible && !$suggestionsSession) {
      const anchor = document.querySelector(".omnibox-wrapper");
      if (anchor instanceof HTMLElement) beginSuggestionsSession(anchor);
    } else if (!visible && $suggestionsSession) {
      endSuggestionsSession();
    }
  });
  onMount(() => {
    window.addEventListener("resize", measure);
    return () => window.removeEventListener("resize", measure);
  });

  // The native popover shows this markup as a presentation-only copy; its clicks come back here as real clicks.
  function host(node: HTMLElement) {
    if (!native) return {};
    const stop = mountPanelHost(node, true, {
      id: "suggestions",
      sessionStore: suggestionsSession,
      close: async () => { closeSuggestions(); },
      set: setSuggestionsPopover,
      listen: onSuggestionsPopoverAction,
    });
    return { destroy: stop };
  }

  function label(item: Suggestion): string {
    if (item.kind === "search") return $t("titlebar.suggest_search", { engine });
    if (item.kind === "navigate") return $t("titlebar.suggest_go");
    if (item.kind === "bookmark") return $t("titlebar.suggest_bookmark");
    if (item.kind === "web") return $t("titlebar.suggest_web");
    return $t("titlebar.suggest_history");
  }
</script>

{#if present}
  <!-- In the app this wrapper keeps the real markup off screen; only the inner element is copied into the popover. -->
  <div class:address-suggestions-controller={native} aria-hidden={native ? "true" : undefined}>
  <div class="address-suggestions" use:host>
    {#if visible}
      <div
        id="address-suggestions-list"
        class="suggestion-panel"
        role="listbox"
        aria-label={$t("titlebar.suggestions_label")}
        style:margin-left={native ? undefined : `${left}px`}
        style:width={native ? undefined : `${width}px`}
        style:min-height={native ? undefined : `${$suggestionState.rows * ROW_HEIGHT + 12}px`}
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
    {/if}
  </div>
  </div>
{/if}
