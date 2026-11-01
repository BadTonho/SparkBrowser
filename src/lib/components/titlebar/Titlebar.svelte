<script lang="ts">
  import {
    addressInput,
    currentUrl,
    tabs,
    isSidebarOpen,
    isLoading,
    toggleSidebar,
    submitNavigation,
    triggerBack,
    triggerForward,
    triggerReload,
  } from "../../stores/browserStore";
  import {
    bookmarks,
    toggleBookmarkForUrl,
    openHistoryDrawer,
  } from "../../stores/persistenceStore";
  import { openDownloadsDrawer } from "../../stores/downloadsStore";
  import { openCommandBar } from "../../stores/commandBarStore";
  import {
    isZenMode,
    toggleZenMode,
    openSettingsModal,
    tabLayout,
  } from "../../stores/settingsStore";
  import { privacyStats, flipAdblockShield } from "../../stores/privacyStore";
  import { workspaces, activeWorkspaceId } from "../../stores/workspaceStore";
  import WorkspaceSwitcher from "../workspaces/WorkspaceSwitcher.svelte";
  import {
    minimizeWindow,
    toggleMaximizeWindow,
    closeWindow,
  } from "../../services/tauri-bridge";

  let activeWs = $derived(
    $workspaces.find((w) => w.id === $activeWorkspaceId)
  );

  let isCurrentUrlBookmarked = $derived(
    $bookmarks.some((b) => b.url === $currentUrl)
  );

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      submitNavigation($addressInput);
    }
  }

  function handleFocus(event: FocusEvent) {
    const target = event.currentTarget as HTMLInputElement;
    target?.select();
  }

  function handleBookmarkToggle() {
    const activeTab = $tabs.find((t) => t.is_active);
    const title = activeTab?.title || $currentUrl;
    toggleBookmarkForUrl(title, $currentUrl, "⭐");
  }
</script>

{#if !$isZenMode}
  <header class="titlebar" data-tauri-drag-region>
    <div class="titlebar-left">
      {#if $tabLayout === "vertical"}
        <button
          class="icon-btn sidebar-toggle"
          class:active={$isSidebarOpen}
          onclick={toggleSidebar}
          title="Toggle Sidebar (Ctrl+B)"
          aria-label="Toggle Sidebar"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="3" />
            <line x1="9" y1="3" x2="9" y2="21" />
          </svg>
        </button>
      {/if}

      {#if $tabLayout === "vertical"}
        <div class="brand" data-tauri-drag-region>
          <span class="brand-dot"></span>
          <span class="brand-name" data-tauri-drag-region>Veloce</span>
          {#if activeWs}
            <span class="ws-badge">{activeWs.icon} {activeWs.name}</span>
          {/if}
        </div>
      {/if}

      {#if $tabLayout === "horizontal"}
        <WorkspaceSwitcher horizontal />
      {/if}
    </div>

    <!-- Centered Omnibox / Search & URL Bar with Navigation Controls to the left -->
    <div class="titlebar-center">
      <div class="nav-controls">
        <button
          class="nav-btn"
          onclick={triggerBack}
          title="Back (Alt+Left)"
          aria-label="Back"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M15 18l-6-6 6-6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>

        <button
          class="nav-btn"
          onclick={triggerForward}
          title="Forward (Alt+Right)"
          aria-label="Forward"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M9 18l6-6-6-6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>

        <button
          class="nav-btn"
          class:spinning={$isLoading}
          onclick={triggerReload}
          title="Reload (Ctrl+R)"
          aria-label="Reload"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M21 12a9 9 0 1 1-2.64-6.36" stroke-linecap="round" />
            <polyline points="21 3 21 9 15 9" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </div>

      <div class="omnibox-wrapper" class:loading={$isLoading}>
        <span class="omnibox-icon">🔒</span>
        <input
          class="omnibox-input"
          type="text"
          bind:value={$addressInput}
          onkeydown={handleKeydown}
          onfocus={handleFocus}
          placeholder="Search DuckDuckGo or enter URL..."
          spellcheck="false"
        />
        <button
          class="star-btn"
          class:bookmarked={isCurrentUrlBookmarked}
          onclick={handleBookmarkToggle}
          title={isCurrentUrlBookmarked ? "Remove Bookmark" : "Bookmark this Page"}
          aria-label="Toggle Bookmark"
        >
          {isCurrentUrlBookmarked ? "★" : "☆"}
        </button>
        <button
          class="go-btn"
          onclick={() => submitNavigation($addressInput)}
          title="Navigate"
          aria-label="Navigate"
        >
          ↵
        </button>
      </div>
    </div>

    <!-- Top-Right Controls: Quick Tools, Settings & Window Controls -->
    <div class="titlebar-controls">
      <button
        class="icon-btn"
        onclick={() => openCommandBar()}
        title="Open Command Bar (Ctrl+K)"
        aria-label="Command Bar"
      >
        ⚡
      </button>

      <button
        class="icon-btn"
        onclick={openHistoryDrawer}
        title="Browsing History (Ctrl+H)"
        aria-label="Browsing History"
      >
        🕒
      </button>

      <button
        class="icon-btn"
        onclick={openDownloadsDrawer}
        title="Download Manager (Ctrl+J)"
        aria-label="Downloads"
      >
        📥
      </button>

      <button
        class="icon-btn shield-btn"
        class:shield-on={$privacyStats.shield_enabled}
        onclick={flipAdblockShield}
        title={$privacyStats.shield_enabled
          ? `Rust AdBlock Active (${$privacyStats.blocked_count} blocked) — Click to pause`
          : "Rust AdBlock Paused — Click to enable"}
        aria-label="Toggle AdBlock Shield"
      >
        <span>🛡️</span>
        <span class="shield-dot"></span>
      </button>

      <button
        class="icon-btn"
        onclick={openSettingsModal}
        title="Settings & Shortcuts (Ctrl+,)"
        aria-label="Settings"
      >
        ⚙️
      </button>

      <button
        class="icon-btn"
        onclick={toggleZenMode}
        title="Enter Zen Focus Mode (Alt+Z)"
        aria-label="Zen Mode"
      >
        🧘
      </button>

      {#if $tabLayout === "vertical"}
        <div class="divider"></div>

        <button class="win-btn" onclick={minimizeWindow} title="Minimize" aria-label="Minimize">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="2" y="5.5" width="8" height="1" fill="currentColor" />
          </svg>
        </button>
        <button class="win-btn" onclick={toggleMaximizeWindow} title="Maximize" aria-label="Maximize">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect
              x="2.5"
              y="2.5"
              width="7"
              height="7"
              rx="1"
              stroke="currentColor"
              stroke-width="1.2"
              fill="none"
            />
          </svg>
        </button>
        <button class="win-btn close-btn" onclick={closeWindow} title="Close" aria-label="Close">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path
              d="M3 3L9 9M9 3L3 9"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linecap="round"
            />
          </svg>
        </button>
      {/if}
    </div>
  </header>
{/if}

<style>
  .titlebar {
    height: 40px;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 0 10px;
    background: var(--bg-glass-primary);
    border-bottom: 1px solid var(--border-subtle);
    backdrop-filter: blur(20px);
    z-index: 20;
    gap: 10px;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12.5px;
    font-weight: 600;
    letter-spacing: 0.3px;
    color: var(--text-secondary);
  }

  .brand-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent-gradient);
    box-shadow: 0 0 10px var(--accent-primary);
  }

  .ws-badge {
    font-size: 10.5px;
    padding: 2px 7px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
  }

  .titlebar-center {
    flex: 1;
    min-width: 0;
    max-width: 980px;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 6px;
  }

  .nav-controls {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .nav-btn {
    width: 27px;
    height: 27px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid transparent;
    font-size: 12px;
    transition:
      background 0.18s ease,
      color 0.18s ease,
      border-color 0.18s ease,
      transform 0.15s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .nav-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
    border-color: var(--border-subtle);
    transform: translateY(-1px);
  }

  .nav-btn:active {
    transform: scale(0.94);
  }

  .nav-btn.spinning svg {
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .omnibox-wrapper {
    flex: 1;
    width: 100%;
    min-width: 0;
    height: 28px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.2);
    transition:
      border-color 0.2s ease,
      background 0.2s ease,
      box-shadow 0.2s ease,
      transform 0.16s ease;
  }

  .omnibox-wrapper:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.18);
  }

  .omnibox-wrapper:focus-within {
    background: rgba(18, 18, 18, 0.95);
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 24%, transparent);
    transform: translateY(-1px);
  }

  .omnibox-wrapper.loading {
    border-color: var(--accent-primary);
    box-shadow: 0 0 12px color-mix(in srgb, var(--accent-primary) 30%, transparent);
  }

  .omnibox-icon {
    font-size: 11px;
    opacity: 0.75;
    flex-shrink: 0;
  }

  .omnibox-input {
    flex: 1;
    width: 100%;
    height: 100%;
    font-size: 12px;
    color: var(--text-primary);
  }

  .omnibox-input::placeholder {
    color: var(--text-muted);
  }

  .star-btn {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    font-size: 13px;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: color 0.15s ease, transform 0.15s ease;
  }

  .star-btn:hover {
    color: #fbbf24;
    transform: scale(1.15);
  }

  .star-btn.bookmarked {
    color: #fbbf24;
  }

  .go-btn {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--accent-primary) 18%, transparent);
    color: var(--accent-primary);
    font-size: 11px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: background 0.15s ease, color 0.15s ease, transform 0.15s ease;
  }

  .go-btn:hover {
    background: var(--accent-primary);
    color: var(--accent-foreground);
    transform: scale(1.08);
  }

  .titlebar-controls {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-shrink: 0;
  }

  .icon-btn {
    position: relative;
    width: 27px;
    height: 26px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    color: var(--text-secondary);
    border: 1px solid transparent;
    flex-shrink: 0;
    transition:
      background 0.18s ease,
      color 0.18s ease,
      border-color 0.18s ease,
      transform 0.15s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .icon-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
    transform: scale(1.05);
  }

  .icon-btn.active {
    color: var(--accent-primary);
    background: color-mix(in srgb, var(--accent-primary) 16%, transparent);
  }

  .shield-btn {
    opacity: 0.65;
  }

  .shield-btn.shield-on {
    opacity: 1;
    background: rgba(34, 197, 94, 0.12);
    border-color: rgba(34, 197, 94, 0.35);
  }

  .shield-dot {
    position: absolute;
    bottom: 2px;
    right: 2px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--text-muted);
    transition: background 0.2s ease, box-shadow 0.2s ease;
  }

  .shield-btn.shield-on .shield-dot {
    background: #22c55e;
    box-shadow: 0 0 6px #22c55e;
  }

  .divider {
    width: 1px;
    height: 16px;
    background: var(--border-subtle);
    margin: 0 3px;
  }

  .win-btn {
    width: 34px;
    height: 26px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    transition: background 0.15s ease, color 0.15s ease;
  }

  .win-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
  }

  .win-btn.close-btn:hover {
    background: #e81123;
    color: #ffffff;
  }
</style>
