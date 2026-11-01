<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import {
    isSettingsModalOpen,
    closeSettingsModal,
    isZenMode,
    toggleZenMode,
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
    { keys: "Ctrl + 1..9", desc: "Switch to Tab 1–9 (Ctrl+9: Last Tab)" },
    { keys: "Hold Alt (2 sec)", desc: "Open the workspace picker" },
    { keys: "Alt + 1..9", desc: "Switch workspaces by list order and show a confirmation" },
    { keys: "Alt + ↑ / ↓", desc: "Move Active Tab Up / Down" },
    { keys: "Ctrl + T / Ctrl + K", desc: "Open Floating Command Bar" },
    { keys: "Ctrl + L", desc: "Edit Current URL in Command Bar" },
    { keys: "Ctrl + B", desc: "Toggle Sidebar (Vertical Layout)" },
    { keys: "Alt + Z", desc: "Toggle Fullscreen Zen / Focus Mode" },
    { keys: "Ctrl + H", desc: "Open SQLite Browsing History" },
    { keys: "Ctrl + J", desc: "Open Download Manager" },
    { keys: "Ctrl + ,", desc: "Open Settings & Shortcuts Panel" },
    { keys: "Ctrl + W", desc: "Close Active Tab" },
    { keys: "Ctrl + R", desc: "Reload Active Tab" },
  ];

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
              onclick={() => setActiveSettingsCategory(category.id)}
            >
              <span class="category-icon" aria-hidden="true">{category.icon}</span>
              <span>{category.label}</span>
            </button>
          {/each}
        </nav>

        <main class="settings-content">
          {#if $activeSettingsCategory === "appearance_layout"}
            <section class="setting-group" aria-labelledby="appearance-heading">
              <h2 class="group-title" id="appearance-heading">🖥️ Appearance & Layout</h2>

              <div class="setting-row">
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

              <div class="setting-row">
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
              <div class="setting-row">
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

              <div class="setting-row">
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

              <div class="setting-row">
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
                  <div class="shortcut-row">
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
    background: rgba(8, 10, 16, 0.62);
    backdrop-filter: blur(16px);
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
    background: rgba(18, 22, 34, 0.96);
    border: 1px solid rgba(110, 142, 251, 0.38);
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
    justify-content: space-between;
  }

  .settings-title {
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .close-btn {
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
    border-color: rgba(110, 142, 251, 0.42);
    background: rgba(110, 142, 251, 0.16);
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
    color: #fff;
  }

  .timer-pills {
    display: flex;
    flex: 0 0 auto;
    gap: 4px;
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
    color: #fff;
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
  }

  @media (max-width: 560px) {
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
    .engine-label {
      align-self: flex-start;
    }
  }
</style>
