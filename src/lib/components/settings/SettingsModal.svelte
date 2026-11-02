<script lang="ts">
import "./settings.css";
import { normalizeSearch as normalizeSearchText } from "../../services/command-ranking.js";
import { createShortcutRecorder } from "./shortcut-recorder.svelte";
import AppearanceSettings from "./AppearanceSettings.svelte";
import UpdateSettings from "./UpdateSettings.svelte";
import NavigationSettings from "./NavigationSettings.svelte";
import PerformanceSettings from "./PerformanceSettings.svelte";
import ShortcutSettings from "./ShortcutSettings.svelte";
  import DiagnosticsSettings from "./DiagnosticsSettings.svelte";
import { focusDialog } from "../../actions/focusDialog";
import { onDestroy, tick } from "svelte";
import { fade, fly } from "svelte/transition";
import { cubicOut } from "svelte/easing";
import {
  isSettingsModalOpen,
  closeSettingsModal,
  activeSettingsCategory,
  setActiveSettingsCategory,
  type SettingsCategory,
} from "../../stores/settingsStore";
import WorkspaceSettings from "./WorkspaceSettings.svelte";

import {
  SHORTCUT_DEFINITIONS,
  formatShortcut,
  getShortcutBinding,
  keyboardShortcuts,
  recordingShortcutId,
} from "../../stores/shortcutsStore";

import { categories, searchableSettings, type SearchEntry } from "./settings-catalog";
let searchQuery = $state("");
let highlightedTargetTimer: ReturnType<typeof setTimeout> | undefined;
let highlightedTarget: HTMLElement | undefined;
const hasSearchQuery = $derived(searchQuery.trim().length > 0);
const searchResults = $derived.by(() => {
  if(!$isSettingsModalOpen) return [];
  const tokens = normalizeSearchText(searchQuery).trim().split(/\s+/).filter(Boolean);
  if(tokens.length === 0) return [];

  const entries = searchableSettings.map((entry) => {
    const shortcutId = entry.targetId.startsWith("shortcut-")
      ? entry.targetId.slice("shortcut-".length)
      : undefined;
    if(!shortcutId) return entry;
    const definition = SHORTCUT_DEFINITIONS.find((item) => item.id === shortcutId);
    const binding = getShortcutBinding($keyboardShortcuts, shortcutId);
    return {
      ...entry,
      description: definition && binding ? formatShortcut(binding, definition.sequence) : entry.description,
    };
  });

  return entries.filter((entry) => {
    const category = categories.find((item) => item.id === entry.category);
    const searchableText = normalizeSearchText(
      `${entry.title} ${entry.description} ${entry.keywords ?? ""} ${category?.label ?? ""}`
    );
    return tokens.every((token) => searchableText.includes(token));
  });
});

function selectCategory(category: SettingsCategory): void {
  searchQuery = "";
  recordingShortcutId.set(null);
  setActiveSettingsCategory(category);
}

function clearSearch(): void {
  searchQuery = "";
  document.getElementById("settings-search-input")?.focus();
}

function handleSearchKeydown(event: KeyboardEvent): void {
  if(event.key === "ArrowDown" && searchResults.length > 0) {
    event.preventDefault();
    document.querySelector<HTMLButtonElement>(".settings-search-result")?.focus();
  } else if(event.key === "Enter" && searchResults.length > 0) {
    event.preventDefault();
    void selectSearchResult(searchResults[0]);
  }
}

async function selectSearchResult(entry: SearchEntry): Promise<void> {
  searchQuery = "";
  recordingShortcutId.set(null);
  setActiveSettingsCategory(entry.category);
  await tick();

  const target = document.getElementById(entry.targetId);
  if(!target) return;

  const focusTarget =
    target.querySelector<HTMLElement>("button, input, select, [tabindex='0']") ?? target;
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  target.scrollIntoView({ behavior: reduceMotion ? "auto" : "smooth", block: "center" });
  focusTarget.focus({ preventScroll: true });
  highlightedTarget?.classList.remove("search-target-highlight");
  target.classList.add("search-target-highlight");
  highlightedTarget = target;

  if(highlightedTargetTimer) clearTimeout(highlightedTargetTimer);
  highlightedTargetTimer = setTimeout(() => {
    target.classList.remove("search-target-highlight");
    highlightedTarget = undefined;
    highlightedTargetTimer = undefined;
  }, 1500);
}

$effect(() => {
  if($isSettingsModalOpen) searchQuery = "";
  else recordingShortcutId.set(null);
});

onDestroy(() => {
  recordingShortcutId.set(null);
  if(highlightedTargetTimer) clearTimeout(highlightedTargetTimer);
  highlightedTarget?.classList.remove("search-target-highlight");
});

const recorder = createShortcutRecorder();
function handleDialogKeydown(event: KeyboardEvent): void {
  if(recorder.keyboard(event)) return;
  if(event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    closeSettingsModal();
    return;
  }
  event.stopPropagation();
}

function handleWindowKeydown(event: KeyboardEvent): void {
  if(recorder.keyboard(event)) return;
  if($isSettingsModalOpen && event.key === "Escape") {
    event.preventDefault();
    closeSettingsModal();
  }
}

function handleWindowMousedown(event: MouseEvent): void {
  recorder.mouse(event);
}
</script>

<svelte:window onkeydown={handleWindowKeydown} onmousedown={handleWindowMousedown} />

{#if $isSettingsModalOpen}
  <div
    class="settings-backdrop"
    transition:fade={{ duration: 150 }}
    onclick={() => closeSettingsModal()}
    role="presentation"
  >
    <div
      class="settings-modal"
      in:fly={{ y: 16, duration: 220, easing: cubicOut }}
      out:fade={{ duration: 130 }}
      onclick={(event) => event.stopPropagation()}
      onkeydown={handleDialogKeydown}
      use:focusDialog={{ onEscape: closeSettingsModal, ignoreEscape: () => Boolean($recordingShortcutId) }}
      role="dialog"
      aria-modal="true"
      aria-label="Browser Settings"
      tabindex="-1"
    >
      <div class="settings-header">
        <span class="settings-title">⚙️ Browser Settings</span>
        <div class="settings-search">
          <span class="search-icon" aria-hidden="true">⌕</span>
          <input
            id="settings-search-input"
            type="search"
            bind:value={searchQuery}
            onkeydown={handleSearchKeydown}
            placeholder="Search settings..."
            aria-label="Search all settings"
            autocomplete="off"
            spellcheck="false"
          />
          {#if hasSearchQuery}
            <button
              class="clear-search-btn"
              type="button"
              onclick={clearSearch}
              aria-label="Clear settings search"
            >
              ×
            </button>
          {/if}
        </div>
        <button
          class="close-btn"
          aria-label="Close settings"
          onclick={() => closeSettingsModal()}
        >
          ×
        </button>
      </div>

      <div class="settings-layout">
        <nav class="category-nav" aria-label="Settings categories">
          {#each categories as category (category.id)}
            <button
              data-testid={`settings-${category.id}`} class="category-button"
              class:active={$activeSettingsCategory === category.id}
              aria-pressed={$activeSettingsCategory === category.id}
              onclick={() => selectCategory(category.id)}
            >
              <span class="category-icon" aria-hidden="true">{category.icon}</span>
              <span>{category.label}</span>
            </button>
          {/each}
        </nav>

        <main class="settings-content" aria-live={hasSearchQuery ? "polite" : undefined}>
          {#if hasSearchQuery}
            <section class="search-results-panel" aria-labelledby="search-results-heading">
              <div class="search-results-heading">
                <h2 class="group-title" id="search-results-heading">Search Results</h2>
                <span class="search-result-count">{searchResults.length}</span>
              </div>
              {#if searchResults.length > 0}
                <div class="search-results-list">
                  {#each searchResults as result (result.id)}
                    {@const category = categories.find((item) => item.id === result.category)}
                    <button
                      class="settings-search-result"
                      onclick={() => selectSearchResult(result)}
                    >
                      <span class="search-result-copy">
                        <span class="search-result-title">{result.title}</span>
                        <span class="search-result-description">{result.description}</span>
                      </span>
                      <span class="search-result-category">
                        <span aria-hidden="true">{category?.icon}</span>
                        {category?.label}
                      </span>
                    </button>
                  {/each}
                </div>
              {:else}
                <p class="search-empty-state" role="status">
                  No settings found for “{searchQuery}”. Try another search.
                </p>
              {/if}
            </section>
          {:else if $activeSettingsCategory === "appearance_layout"}
            <AppearanceSettings />
          {:else if $activeSettingsCategory === "search_navigation"}
            <NavigationSettings />
          {:else if $activeSettingsCategory === "workspaces"}
            <WorkspaceSettings />
          {:else if $activeSettingsCategory === "performance_privacy"}
            <PerformanceSettings />
          {:else if $activeSettingsCategory === "shortcuts"}
            <ShortcutSettings error={recorder.error} onStart={recorder.start} />
          {:else if $activeSettingsCategory === "diagnostics"}
            <DiagnosticsSettings />
          {:else if $activeSettingsCategory === "updates"}
            <UpdateSettings />
          {/if}
        </main>
      </div>
    </div>
  </div>
{/if}
