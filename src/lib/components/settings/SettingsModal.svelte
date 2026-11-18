<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { t, labelText } from "../../i18n";
import "./settings.css";
import { normalizeSearch as normalizeSearchText } from "../../services/command-ranking.js";
import { createShortcutRecorder } from "./shortcut-recorder.svelte";
import AppearanceSettings from "./AppearanceSettings.svelte";
import BackupSettings from "./BackupSettings.svelte";
import UpdateSettings from "./UpdateSettings.svelte";
import ChangelogSettings from "./ChangelogSettings.svelte";
import NavigationSettings from "./NavigationSettings.svelte";
import PerformanceSettings from "./PerformanceSettings.svelte";
import PrivacySecuritySettings from "./PrivacySecuritySettings.svelte";
import ShortcutSettings from "./ShortcutSettings.svelte";
  import DiagnosticsSettings from "./DiagnosticsSettings.svelte";
import { focusDialog } from "../../actions/focusDialog";
import { isTauriRuntime } from "../../services/bridge/runtime";
import { mountPanelHost } from "../panels/panel-host";
import { settingsOverlaySession, settingsTabSession, closeSettingsTab } from "../../stores/settingsStore";
import { setSettingsOverlay, onSettingsOverlayAction, setSettingsTab, onSettingsTabAction } from "../../services/bridge/settings-overlay";
const native = isTauriRuntime();
function host(node: HTMLElement) {
  if (!native) return focusDialog(node, { onEscape: () => closeSettingsModal(), ignoreEscape: () => Boolean($recordingShortcutId) });
  const tab = $isSettingsModalOpen ? null : $settingsTabSession;
  if (tab) {
    const stop = mountPanelHost(node, true, { id: "settings", independent: true, sessionStore: settingsTabSession, close: () => closeSettingsTab(), set: (visible, session, presentation) => setSettingsTab(tab.tabId, visible, session, presentation, () => !$isSettingsModalOpen && $settingsTabSession?.id === session), listen: callback => onSettingsTabAction(tab.tabId, callback) });
    return { destroy: stop };
  }
  const stop = mountPanelHost(node, true, { id: "settings", sessionStore: settingsOverlaySession, close: (_restore, immediate) => closeSettingsModal(immediate), set: setSettingsOverlay, listen: onSettingsOverlayAction });
  return { destroy: stop };
}
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
// The search starts as a single icon; it opens on click and closes again when it is empty and loses focus.
let searchOpen = $state(false);
let highlightedTargetTimer: ReturnType<typeof setTimeout> | undefined;
let highlightedTarget: HTMLElement | undefined;
const hasSearchQuery = $derived(searchQuery.trim().length > 0);
const searchExpanded = $derived(searchOpen || hasSearchQuery);
const searchResults = $derived.by(() => {
  if(!$isSettingsModalOpen && !$settingsTabSession) return [];
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
      `${$labelText(entry.title)} ${$labelText(entry.description)} ${entry.keywords ?? ""} ${category?.label ?? ""}`
    );
    return tokens.every((token) => searchableText.includes(token));
  });
});

async function openSearch(): Promise<void> {
  searchOpen = true;
  await tick();
  if (!native) document.getElementById("settings-search-input")?.focus();
}

function collapseSearch(): void {
  if (!hasSearchQuery) searchOpen = false;
}

function selectCategory(category: SettingsCategory): void {
  searchQuery = "";
  searchOpen = false;
  recordingShortcutId.set(null);
  setActiveSettingsCategory(category);
}

function clearSearch(): void {
  searchQuery = "";
  if (!native) document.getElementById("settings-search-input")?.focus();
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
  searchOpen = false;
  recordingShortcutId.set(null);
  setActiveSettingsCategory(entry.category);
  await tick();

  const target = document.getElementById(entry.targetId);
  if(!target) return;

  const focusTarget =
    target.querySelector<HTMLElement>("button, input, select, [tabindex='0']") ?? target;
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  target.scrollIntoView({ behavior: reduceMotion ? "auto" : "smooth", block: "center" });
  if (!native) focusTarget.focus({ preventScroll: true });
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
  if($isSettingsModalOpen || $settingsTabSession) { searchQuery = ""; searchOpen = false; }
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
  if(event.key === "Escape" && $isSettingsModalOpen) {
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

{#if $isSettingsModalOpen || $settingsTabSession}
  {#key $isSettingsModalOpen ? "popup" : $settingsTabSession?.id}
  <div
    class="settings-backdrop"
    class:settings-controller={native}
    transition:fade={{ duration: native || window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 130 }}
    onclick={() => closeSettingsModal()}
    role="presentation"
  >
    <div
      class="settings-modal"
      in:fly={{ y: 12, duration: native || window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 180, easing: cubicOut }}
      out:fly={{ y: 8, duration: native || window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 130 }}
      onclick={(event) => event.stopPropagation()}
      onkeydown={handleDialogKeydown}
      use:host
      role={$isSettingsModalOpen ? "dialog" : "main"}
      aria-modal={$isSettingsModalOpen ? "true" : undefined}
      aria-label={$t("settings.browser_settings_6c1e")}
      tabindex="-1"
    >
      <div class="settings-header">
        <span class="settings-title"><AppIcon name="settings" /> {$t("settings.browser_settings_59f9")}</span>
        <div class="settings-search" class:expanded={searchExpanded}>
          <button
            id="settings-search-open"
            class="search-open"
            type="button"
            aria-label={$t("settings.search_all_settings_b98f")}
            aria-expanded={searchExpanded}
            onclick={openSearch}
          >
            <span class="search-open-icon" aria-hidden="true">⌕</span>
          </button>
          <div class="search-box">
            <span class="search-icon" aria-hidden="true">⌕</span>
            <input
              id="settings-search-input"
              type="search"
              bind:value={searchQuery}
              onkeydown={handleSearchKeydown}
              onblur={collapseSearch}
              placeholder={$t("settings.search_settings_6801")}
              aria-label={$t("settings.search_all_settings_b98f")}
              autocomplete="off"
              spellcheck="false"
            />
            {#if hasSearchQuery}
              <button
                class="clear-search-btn"
                type="button"
                onclick={clearSearch}
                aria-label={$t("settings.clear_settings_search_63c6")}
              >
                ×
              </button>
            {/if}
          </div>
        </div>
        <button
          class="close-btn"
          aria-label={$t("settings.close_settings_0ccf")}
          onclick={() => $isSettingsModalOpen ? closeSettingsModal() : closeSettingsTab()}
        >
          ×
        </button>
      </div>

      <div class="settings-layout">
        <nav class="category-nav" aria-label={$t("settings.settings_categories_b9ed")}>
          {#each categories as category (category.id)}
            <button
              data-testid={`settings-${category.id}`} class="category-button"
              class:active={$activeSettingsCategory === category.id}
              aria-pressed={$activeSettingsCategory === category.id}
              onclick={() => selectCategory(category.id)}
            >
              <span class="category-icon" aria-hidden="true"><AppIcon name={category.icon} /></span>
              <span>{$labelText(category.label)}</span>
            </button>
          {/each}
        </nav>

        <main class="settings-content" aria-live={hasSearchQuery ? "polite" : undefined}>
          {#if hasSearchQuery}
            <section class="search-results-panel" aria-labelledby="search-results-heading">
              <div class="search-results-heading">
                <h2 class="group-title" id="search-results-heading">{$t("settings.search_results_3941")}</h2>
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
                        <span class="search-result-title">{$labelText(result.title)}</span>
                        <span class="search-result-description">{$labelText(result.description)}</span>
                      </span>
                      <span class="search-result-category">
                        <span aria-hidden="true"><AppIcon name={category?.icon ?? ""} /></span>
                        {$labelText(category?.label ?? "")}
                      </span>
                    </button>
                  {/each}
                </div>
              {:else}
                <p class="search-empty-state" role="status">
                  {$t("settings.no_settings_found_for_753d")}{searchQuery}{$t("settings.try_another_search_1064")}
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
          {:else if $activeSettingsCategory === "privacy_security"}
            <PrivacySecuritySettings />
          {:else if $activeSettingsCategory === "shortcuts"}
            <ShortcutSettings error={recorder.error} onStart={recorder.start} />
          {:else if $activeSettingsCategory === "diagnostics"}
            <DiagnosticsSettings />
          {:else if $activeSettingsCategory === "data_backup"}
            <BackupSettings />
          {:else if $activeSettingsCategory === "updates"}
            <UpdateSettings />
          {:else if $activeSettingsCategory === "changelog"}
            <ChangelogSettings />
          {/if}
        </main>
      </div>
    </div>
  </div>
  {/key}
{/if}
