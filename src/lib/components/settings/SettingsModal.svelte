<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import {
    isSettingsModalOpen,
    closeSettingsModal,
    isZenMode,
    toggleZenMode,
    overlayBlurPx,
    setOverlayBlurPx,
    autoHibernateMinutes,
    defaultSearchEngine,
    tabLayout,
    setTabLayout,
    activeSettingsCategory,
    setActiveSettingsCategory,
    type SettingsCategory,
  } from "../../stores/settingsStore";
  import WorkspaceSettings from "./WorkspaceSettings.svelte";
  import { privacyStats, flipAdblockShield } from "../../stores/privacyStore";

  const categories: {
    id: SettingsCategory;
    icon: string;
    label: string;
  }[] = [
    { id: "appearance_layout", icon: "🖥️", label: "Appearance & Layout" },
    { id: "search_navigation", icon: "🔎", label: "Search & Navigation" },
    { id: "workspaces", icon: "🎨", label: "Workspaces & Theme" },
    { id: "performance_privacy", icon: "🚀", label: "Performance & Privacy" },
    { id: "shortcuts", icon: "⌨️", label: "Keyboard Shortcuts" },
  ];

  const shortcutsList = [
    { id: "shortcut-switch-tabs", keys: "Ctrl + 1..9", desc: "Switch to Tab 1–9 (Ctrl+9: Last Tab)" },
    { id: "shortcut-workspace-picker", keys: "Hold Alt (0.4 sec)", desc: "Open the workspace picker" },
    { id: "shortcut-switch-workspaces", keys: "Alt + 1..9", desc: "Switch workspaces by list order and show a confirmation" },
    { id: "shortcut-move-tab", keys: "Alt + ↑ / ↓", desc: "Move Active Tab Up / Down" },
    { id: "shortcut-command-bar", keys: "Ctrl + T / Ctrl + K", desc: "Open Floating Command Bar" },
    { id: "shortcut-edit-url", keys: "Ctrl + L", desc: "Edit Current URL in Command Bar" },
    { id: "shortcut-sidebar", keys: "Ctrl + B", desc: "Toggle Sidebar (Vertical Layout)" },
    { id: "shortcut-zen", keys: "Alt + Z", desc: "Toggle Fullscreen Zen / Focus Mode" },
    { id: "shortcut-history", keys: "Ctrl + H", desc: "Open SQLite Browsing History" },
    { id: "shortcut-downloads", keys: "Ctrl + J", desc: "Open Download Manager" },
    { id: "shortcut-settings", keys: "Ctrl + ,", desc: "Open Settings & Shortcuts Panel" },
    { id: "shortcut-close-tab", keys: "Ctrl + W", desc: "Close Active Tab" },
    { id: "shortcut-reload-tab", keys: "Ctrl + R", desc: "Reload Active Tab" },
  ];

  type SearchEntry = {
    id: string;
    title: string;
    description: string;
    category: SettingsCategory;
    targetId: string;
    keywords?: string;
  };

  const searchableSettings: SearchEntry[] = [
    {
      id: "browser-layout",
      title: "Browser Layout",
      description: "Choose a horizontal browser layout or a vertical sidebar layout.",
      category: "appearance_layout",
      targetId: "setting-browser-layout",
      keywords: "horizontal vertical tabs sidebar workspaces bookmarks",
    },
    {
      id: "background-blur",
      title: "Background Blur",
      description: "Control the blur intensity behind browser overlays.",
      category: "appearance_layout",
      targetId: "setting-background-blur",
      keywords: "overlay intensity transparency glass 0 24 px",
    },
    {
      id: "zen-mode",
      title: "Zen / Focus Mode",
      description: "Hide browser controls so websites use the full viewport.",
      category: "appearance_layout",
      targetId: "setting-zen-mode",
      keywords: "fullscreen focus alt z",
    },
    {
      id: "default-search-engine",
      title: "Default Search Engine",
      description: "Used when typing search queries in the Omnibox.",
      category: "search_navigation",
      targetId: "setting-default-search-engine",
      keywords: "search omnibox web duckduckgo",
    },
    {
      id: "workspace-profile",
      title: "Workspace Identity & Theme",
      description: "Choose a workspace to edit its name, emoji, colors, or manage profiles.",
      category: "workspaces",
      targetId: "setting-workspace-profiles",
      keywords: "workspaces personal work study gaming name rename emoji icon theme primary secondary accent colors create delete",
    },
    {
      id: "adblock-shield",
      title: "Native Rust AdBlock Shield",
      description: "Block ad domains and trackers at the network and DOM layers.",
      category: "performance_privacy",
      targetId: "setting-adblock-shield",
      keywords: "privacy ads trackers block protection",
    },
    {
      id: "auto-hibernate",
      title: "Auto-Hibernate Inactive Tabs",
      description: "Suspend background WebViews after an idle threshold.",
      category: "performance_privacy",
      targetId: "setting-auto-hibernate",
      keywords: "performance memory ram 5m 10m 30m",
    },
    ...shortcutsList.map((shortcut) => ({
      id: shortcut.id,
      title: shortcut.desc,
      description: shortcut.keys,
      category: "shortcuts" as const,
      targetId: shortcut.id,
      keywords: "keyboard shortcut",
    })),
  ];

  let searchQuery = $state("");
  let highlightedTargetTimer: ReturnType<typeof setTimeout> | undefined;
  let highlightedTarget: HTMLElement | undefined;
  const hasSearchQuery = $derived(searchQuery.trim().length > 0);
  const searchResults = $derived.by(() => {
    const tokens = normalizeSearchText(searchQuery).trim().split(/\s+/).filter(Boolean);
    if (tokens.length === 0) return [];

    return searchableSettings.filter((entry) => {
      const category = categories.find((item) => item.id === entry.category);
      const searchableText = normalizeSearchText(
        `${entry.title} ${entry.description} ${entry.keywords ?? ""} ${category?.label ?? ""}`
      );
      return tokens.every((token) => searchableText.includes(token));
    });
  });

  function normalizeSearchText(value: string): string {
    return value
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase();
  }

  function selectCategory(category: SettingsCategory): void {
    searchQuery = "";
    setActiveSettingsCategory(category);
  }

  function clearSearch(): void {
    searchQuery = "";
    document.getElementById("settings-search-input")?.focus();
  }

  function handleSearchKeydown(event: KeyboardEvent): void {
    if (event.key === "ArrowDown" && searchResults.length > 0) {
      event.preventDefault();
      document.querySelector<HTMLButtonElement>(".settings-search-result")?.focus();
    } else if (event.key === "Enter" && searchResults.length > 0) {
      event.preventDefault();
      void selectSearchResult(searchResults[0]);
    }
  }

  async function selectSearchResult(entry: SearchEntry): Promise<void> {
    searchQuery = "";
    setActiveSettingsCategory(entry.category);
    await tick();

    const target = document.getElementById(entry.targetId);
    if (!target) return;

    const focusTarget =
      target.querySelector<HTMLElement>("button, input, select, [tabindex='0']") ?? target;
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    target.scrollIntoView({ behavior: reduceMotion ? "auto" : "smooth", block: "center" });
    focusTarget.focus({ preventScroll: true });
    highlightedTarget?.classList.remove("search-target-highlight");
    target.classList.add("search-target-highlight");
    highlightedTarget = target;

    if (highlightedTargetTimer) clearTimeout(highlightedTargetTimer);
    highlightedTargetTimer = setTimeout(() => {
      target.classList.remove("search-target-highlight");
      highlightedTarget = undefined;
      highlightedTargetTimer = undefined;
    }, 1500);
  }

  $effect(() => {
    if ($isSettingsModalOpen) searchQuery = "";
  });

  onDestroy(() => {
    if (highlightedTargetTimer) clearTimeout(highlightedTargetTimer);
    highlightedTarget?.classList.remove("search-target-highlight");
  });

  function handleDialogKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      closeSettingsModal();
      return;
    }
    event.stopPropagation();
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if ($isSettingsModalOpen && event.key === "Escape") {
      event.preventDefault();
      closeSettingsModal();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

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
              class="category-button"
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
            <section class="setting-group" aria-labelledby="appearance-heading">
              <h2 class="group-title" id="appearance-heading">🖥️ Appearance & Layout</h2>

              <div class="setting-row" id="setting-browser-layout" tabindex="-1">
                <div class="row-text">
                  <span class="row-label">Browser Layout</span>
                  <span class="row-desc">Choose a horizontal browser layout or a vertical sidebar layout</span>
                </div>
                <div class="timer-pills">
                  <button
                    class="mini-pill"
                    class:selected={$tabLayout === "horizontal"}
                    onclick={() => setTabLayout("horizontal")}
                    title="Tabs at the top, workspaces beside search, and bookmarks below"
                  >
                    ↔ Horizontal
                  </button>
                  <button
                    class="mini-pill"
                    class:selected={$tabLayout === "vertical"}
                    onclick={() => setTabLayout("vertical")}
                    title="Workspaces, bookmarks, and tabs in the sidebar"
                  >
                    ↕ Vertical
                  </button>
                </div>
              </div>

              <div class="setting-row" id="setting-background-blur" tabindex="-1">
                <div class="row-text">
                  <span class="row-label">Background Blur</span>
                  <span class="row-desc">Adjust blur behind browser overlays</span>
                </div>
                <div class="range-control">
                  <input
                    type="range"
                    min="0"
                    max="24"
                    step="1"
                    value={$overlayBlurPx}
                    oninput={(event) => setOverlayBlurPx(Number(event.currentTarget.value))}
                    aria-label="Background blur intensity"
                    aria-valuetext={`${$overlayBlurPx} pixels`}
                  />
                  <output>{ $overlayBlurPx } px</output>
                </div>
              </div>

              <div class="setting-row" id="setting-zen-mode" tabindex="-1">
                <div class="row-text">
                  <span class="row-label">Zen / Focus Mode (Alt + Z)</span>
                  <span class="row-desc">Hides all browser chrome so websites use 100% of the viewport</span>
                </div>
                <button
                  class="pill-action"
                  class:active={$isZenMode}
                  onclick={toggleZenMode}
                >
                  {$isZenMode ? "Active" : "Off"}
                </button>
              </div>
            </section>
          {:else if $activeSettingsCategory === "search_navigation"}
            <section class="setting-group" aria-labelledby="search-heading">
              <h2 class="group-title" id="search-heading">🔎 Search & Navigation</h2>
              <div class="setting-row" id="setting-default-search-engine" tabindex="-1">
                <div class="row-text">
                  <span class="row-label">Default Search Engine</span>
                  <span class="row-desc">Used when typing search queries in the Omnibox</span>
                </div>
                <span class="engine-label">{$defaultSearchEngine}</span>
              </div>
            </section>
          {:else if $activeSettingsCategory === "workspaces"}
            <WorkspaceSettings />
          {:else if $activeSettingsCategory === "performance_privacy"}
            <section class="setting-group" aria-labelledby="performance-heading">
              <h2 class="group-title" id="performance-heading">🚀 Performance & Privacy</h2>

              <div class="setting-row" id="setting-adblock-shield" tabindex="-1">
                <div class="row-text">
                  <span class="row-label">Native Rust AdBlock Shield</span>
                  <span class="row-desc">Blocks ad domains & trackers at the network/DOM layer</span>
                </div>
                <button
                  class="pill-action"
                  class:active={$privacyStats.shield_enabled}
                  onclick={flipAdblockShield}
                >
                  {$privacyStats.shield_enabled ? "Enabled" : "Paused"}
                </button>
              </div>

              <div class="setting-row" id="setting-auto-hibernate" tabindex="-1">
                <div class="row-text">
                  <span class="row-label">Auto-Hibernate Inactive Tabs</span>
                  <span class="row-desc">Suspends background WebViews after idle threshold ({$autoHibernateMinutes} min)</span>
                </div>
                <div class="timer-pills">
                  {#each [5, 10, 30] as mins}
                    <button
                      class="mini-pill"
                      class:selected={$autoHibernateMinutes === mins}
                      aria-pressed={$autoHibernateMinutes === mins}
                      onclick={() => autoHibernateMinutes.set(mins)}
                    >
                      {mins}m
                    </button>
                  {/each}
                </div>
              </div>
            </section>
          {:else if $activeSettingsCategory === "shortcuts"}
            <section class="setting-group" aria-labelledby="shortcuts-heading">
              <h2 class="group-title" id="shortcuts-heading">⌨️ Keyboard Shortcuts</h2>
              <div class="shortcuts-grid">
                {#each shortcutsList as item}
                  <div class="shortcut-row" id={item.id} tabindex="-1">
                    <span class="sc-desc">{item.desc}</span>
                    <kbd class="sc-keys">{item.keys}</kbd>
                  </div>
                {/each}
              </div>
            </section>
          {/if}
        </main>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    z-index: 995;
    background: rgba(8, 8, 8, 0.62);
    backdrop-filter: blur(var(--overlay-backdrop-blur, 16px));
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .settings-modal {
    width: 80vw;
    height: 80dvh;
    max-width: calc(100vw - 32px);
    max-height: calc(100dvh - 32px);
    border-radius: var(--radius-lg);
    background: rgba(18, 18, 18, 0.96);
    border: 1px solid var(--border-glow);
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.65);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .settings-header {
    flex: 0 0 auto;
    padding: 15px 18px;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .settings-title {
    flex: 0 0 auto;
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .settings-search {
    min-width: 0;
    width: min(38vw, 420px);
    height: 34px;
    margin-left: 2px;
    padding: 0 9px;
    display: flex;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.035);
    color: var(--text-muted);
    transition: border-color 0.15s ease, background 0.15s ease;
  }

  .settings-search:focus-within {
    border-color: color-mix(in srgb, var(--accent-primary) 65%, var(--border-subtle));
    background: rgba(255, 255, 255, 0.055);
  }

  .search-icon {
    flex: 0 0 auto;
    font-size: 19px;
    line-height: 1;
  }

  .settings-search input {
    width: 100%;
    min-width: 0;
    height: 100%;
    border: 0;
    outline: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: 12px;
  }

  .settings-search input::placeholder {
    color: var(--text-muted);
  }

  .settings-search input::-webkit-search-cancel-button {
    display: none;
  }

  .clear-search-btn {
    flex: 0 0 auto;
    width: 22px;
    height: 22px;
    border-radius: 5px;
    color: var(--text-secondary);
    font-size: 17px;
    line-height: 1;
  }

  .clear-search-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
  }

  .close-btn {
    margin-left: auto;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    font-size: 19px;
  }

  .close-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
  }

  .settings-layout {
    min-height: 0;
    flex: 1 1 auto;
    display: flex;
    overflow: hidden;
  }

  .category-nav {
    width: 204px;
    flex: 0 0 204px;
    padding: 14px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    border-right: 1px solid var(--border-subtle);
  }

  .category-button {
    width: 100%;
    min-height: 42px;
    padding: 8px 10px;
    display: flex;
    align-items: center;
    gap: 9px;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    text-align: left;
    color: var(--text-secondary);
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1.25;
    transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease;
  }

  .category-button:hover {
    background: rgba(255, 255, 255, 0.045);
    color: var(--text-primary);
  }

  .category-button.active {
    border-color: color-mix(in srgb, var(--accent-primary) 42%, transparent);
    background: color-mix(in srgb, var(--accent-primary) 16%, transparent);
    color: var(--text-primary);
  }

  .category-icon {
    flex: 0 0 auto;
    width: 18px;
    text-align: center;
    font-size: 14px;
  }

  .settings-content {
    min-width: 0;
    min-height: 0;
    flex: 1 1 auto;
    padding: 20px;
    overflow-y: auto;
    overscroll-behavior: contain;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .search-results-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .search-results-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .search-result-count {
    color: var(--text-muted);
    font-size: 10px;
  }

  .search-results-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .settings-search-result {
    min-width: 0;
    width: 100%;
    padding: 10px 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.025);
    text-align: left;
    transition: border-color 0.15s ease, background 0.15s ease;
  }

  .settings-search-result:hover {
    border-color: color-mix(in srgb, var(--accent-primary) 55%, var(--border-subtle));
    background: rgba(255, 255, 255, 0.06);
  }

  .settings-search-result:focus-visible {
    border-color: var(--accent-primary);
    outline: 2px solid var(--accent-primary);
    outline-offset: 2px;
  }

  .search-result-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .search-result-title {
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 650;
  }

  .search-result-description {
    color: var(--text-secondary);
    font-size: 10.5px;
  }

  .search-result-category {
    flex: 0 0 auto;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--text-muted);
    font-size: 10px;
  }

  .search-empty-state {
    margin: 0;
    padding: 18px 12px;
    border: 1px dashed var(--border-subtle);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: 12px;
    text-align: center;
  }

  :global(.search-target-highlight) {
    outline: 2px solid var(--accent-primary);
    outline-offset: 3px;
    border-radius: var(--radius-md);
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .group-title {
    margin: 0;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.7px;
    color: var(--text-muted);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px;
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border-subtle);
  }

  .row-text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .row-label {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .row-desc {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .pill-action {
    flex: 0 0 auto;
    padding: 6px 12px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.08);
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .pill-action.active {
    background: var(--accent-gradient);
    color: var(--accent-foreground);
  }

  .timer-pills {
    display: flex;
    flex: 0 0 auto;
    gap: 4px;
  }

  .range-control {
    min-width: 160px;
    flex: 0 1 280px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .range-control input {
    min-width: 0;
    flex: 1 1 auto;
    accent-color: var(--accent-primary);
    cursor: pointer;
  }

  .range-control output {
    min-width: 42px;
    color: var(--text-secondary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .mini-pill {
    padding: 5px 8px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    font-size: 11px;
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .mini-pill.selected {
    background: var(--accent-primary);
    color: var(--accent-foreground);
  }

  .engine-label {
    flex: 0 0 auto;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--accent-primary);
  }

  .shortcuts-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 6px;
  }

  .shortcut-row {
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.025);
    font-size: 12px;
  }

  .setting-row:focus,
  .shortcut-row:focus {
    outline: none;
  }

  .sc-desc {
    color: var(--text-secondary);
  }

  .sc-keys {
    flex: 0 0 auto;
    padding: 2px 7px;
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-subtle);
    font-size: 10.5px;
    color: var(--text-primary);
  }

  @media (max-width: 720px) {
    .settings-backdrop {
      padding: 16px;
    }

    .category-nav {
      width: 178px;
      flex-basis: 178px;
    }

    .settings-content {
      padding: 16px;
    }

    .settings-search {
      width: auto;
      flex: 1 1 auto;
    }
  }

  @media (max-width: 560px) {
    .settings-header {
      gap: 8px;
      padding-inline: 10px;
    }

    .settings-title {
      max-width: 42%;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-size: 12px;
    }

    .settings-search {
      height: 31px;
      margin-left: 0;
      padding-inline: 7px;
      gap: 5px;
    }

    .settings-search input {
      font-size: 11px;
    }

    .category-nav {
      width: 54px;
      flex-basis: 54px;
      padding-inline: 6px;
    }

    .category-button {
      justify-content: center;
      padding: 8px 4px;
    }

    .category-button > span:last-child {
      display: none;
    }

    .category-icon {
      font-size: 16px;
    }

    .settings-content {
      padding: 14px;
    }

    .setting-row {
      align-items: flex-start;
      flex-direction: column;
    }

    .timer-pills,
    .pill-action,
    .engine-label,
    .range-control {
      align-self: flex-start;
    }

    .range-control {
      width: 100%;
      max-width: 340px;
    }

    .settings-search-result {
      align-items: flex-start;
      flex-direction: column;
      gap: 7px;
    }
  }
</style>
