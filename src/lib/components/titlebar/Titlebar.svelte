<script lang="ts">
  import ZoomControls from "./ZoomControls.svelte";
  import { t } from "../../i18n";
  import { openSitePermissions } from "../../stores/sitePermissionsStore";
  import { openTools } from "../../stores/toolsStore";
  import {
    addressInput,
    isAddressEditing,
    activeNavigation,
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
    openBookmarkEditor,
    openHistoryDrawer,
  } from "../../stores/persistenceStore";
  import { openDownloadsDrawer } from "../../stores/downloadsStore";
  import { openCommandBar } from "../../stores/commandBarStore";
  import {
    isZenMode,
    toggleZenMode,
    openSettingsModal,
    tabLayout,
    showBookmarksBar,
    toggleBookmarksBar,
    workspaceSwitcherDisplay,
  } from "../../stores/settingsStore";
  import { privacyStats, flipAdblockShield } from "../../stores/privacyStore";
  import { workspaces, activeWorkspaceId } from "../../stores/workspaceStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import { defaultSearchEngine } from "../../stores/searchEngineStore";
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
      isAddressEditing.set(false);
      (event.currentTarget as HTMLInputElement).blur();
      void submitNavigation($addressInput);
    }
  }

  function handleFocus(event: FocusEvent) {
    isAddressEditing.set(true);
    const target = event.currentTarget as HTMLInputElement;
    target?.select();
  }

  function handleBookmarkToggle() {
    const activeTab = $tabs.find((t) => t.is_active);
    const title = activeTab?.title || $currentUrl;
    void openBookmarkEditor(title, $currentUrl, "⭐");
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
          title={$t("titlebar.toggle_sidebar_p0_d643", { p0: $keyboardShortcuts && shortcutLabel("sidebar") })}
          aria-label={$t("titlebar.toggle_sidebar_1d4c")}
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

      {#if $tabLayout === "horizontal" && $workspaceSwitcherDisplay === "toolbar"}
        <WorkspaceSwitcher horizontal />
      {/if}
    </div>

    <!-- Centered Omnibox / Search & URL Bar with Navigation Controls to the left -->
    <div class="titlebar-center">
      <div class="nav-controls">
        <button
          class="nav-btn"
          disabled={$activeNavigation?.can_go_back === false}
          onclick={triggerBack}
          title={$t("titlebar.back_p0_f6f0", { p0: $keyboardShortcuts && shortcutLabel("navigate-back") })}
          aria-label={$t("titlebar.back_7690")}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M15 18l-6-6 6-6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>

        <button
          class="nav-btn"
          disabled={$activeNavigation?.can_go_forward === false}
          onclick={triggerForward}
          title={$t("titlebar.forward_p0_5837", { p0: $keyboardShortcuts && shortcutLabel("navigate-forward") })}
          aria-label={$t("titlebar.forward_f1c6")}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M9 18l6-6-6-6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>

        <button
          class="nav-btn"
          class:spinning={$isLoading}
          onclick={triggerReload}
          title={$t("titlebar.reload_p0_ac34", { p0: $keyboardShortcuts && shortcutLabel("reload-tab") })}
          aria-label={$t("titlebar.reload_bdc0")}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
            <path d="M21 12a9 9 0 1 1-2.64-6.36" stroke-linecap="round" />
            <polyline points="21 3 21 9 15 9" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </div>

      <div class="omnibox-wrapper" class:loading={$isLoading} title={$activeNavigation?.error ?? undefined}>
        <button class="omnibox-icon" onclick={openSitePermissions} aria-label={$t("titlebar.site_permissions_a688")} title={$currentUrl.startsWith("https:") ? $t("titlebar.https_connection_site_permissions_9979") : $t("titlebar.unencrypted_connection_site_permissions_7add")}>{$currentUrl.startsWith("https:") ? "\u{1F512}" : "\u26A0"}</button>
        <input
          class="omnibox-input"
          type="text"
          bind:value={$addressInput}
          onkeydown={(event) => { if (event.key === "Escape") { addressInput.set($currentUrl); isAddressEditing.set(false); (event.currentTarget as HTMLInputElement).blur(); } else handleKeydown(event); }}
          onfocus={handleFocus}
          onblur={() => isAddressEditing.set(false)}
          placeholder="{$t("titlebar.search_49c2")} {$defaultSearchEngine} {$t("titlebar.or_enter_url_5dc7")}"
          spellcheck="false"
        />
        <ZoomControls />
        <button
          class="star-btn"
          class:bookmarked={isCurrentUrlBookmarked}
          onclick={handleBookmarkToggle}
          title={isCurrentUrlBookmarked ? $t("titlebar.edit_bookmark_6e59") : $t("titlebar.bookmark_this_page_b12b")}
          aria-label={isCurrentUrlBookmarked ? $t("titlebar.edit_bookmark_6e59") : $t("titlebar.bookmark_this_page_b12b")}
        >
          {isCurrentUrlBookmarked ? "★" : "☆"}
        </button>
        <button
          class="go-btn"
          onclick={() => submitNavigation($addressInput)}
          title={$t("titlebar.navigate_5ea5")}
          aria-label={$t("titlebar.navigate_5ea5")}
        >
          ↵
        </button>
      </div>
    </div>

    <!-- Top-Right Controls: Quick Tools, Settings & Window Controls -->
    <div class="titlebar-controls">
      <button class="icon-btn" onclick={() => openTools()} title={$t("titlebar.tools_ea93")} data-testid="open-tools" aria-label={$t("titlebar.tools_ea93")}>☷</button>
      {#if $tabLayout === "horizontal"}
        <button
          class="icon-btn"
          class:active={$showBookmarksBar}
          onclick={toggleBookmarksBar}
          title={$showBookmarksBar ? $t("titlebar.hide_bookmarks_bar_31b5") : $t("titlebar.show_bookmarks_bar_570b")}
          aria-label={$showBookmarksBar ? $t("titlebar.hide_bookmarks_bar_31b5") : $t("titlebar.show_bookmarks_bar_570b")}
          aria-pressed={$showBookmarksBar}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
            <path d="M4 5.5h16M4 12h16M4 18.5h16" stroke-linecap="round" />
            <path d="M7 3.5v4M17 3.5v4" stroke-linecap="round" />
          </svg>
        </button>
      {/if}

      <button
        class="icon-btn"
        onclick={() => openCommandBar()}
        title={$t("titlebar.open_command_bar_p0_cf83", { p0: $keyboardShortcuts && shortcutLabel("command-bar") })}
        aria-label={$t("titlebar.command_bar_1356")}
      >
        ⚡
      </button>

      <button
        class="icon-btn"
        onclick={openHistoryDrawer}
        title={$t("titlebar.browsing_history_p0_cf44", { p0: $keyboardShortcuts && shortcutLabel("history") })}
        data-testid="open-history" aria-label={$t("titlebar.browsing_history_13cc")}
      >
        🕒
      </button>

      <button
        class="icon-btn"
        onclick={openDownloadsDrawer}
        title={$t("titlebar.download_manager_p0_8620", { p0: $keyboardShortcuts && shortcutLabel("downloads") })}
        data-testid="open-downloads" aria-label={$t("titlebar.downloads_d5fd")}
      >
        📥
      </button>

      <button
        class="icon-btn shield-btn"
        class:shield-on={$privacyStats.shield_enabled}
        onclick={flipAdblockShield}
        title={$privacyStats.shield_enabled
          ? $t("titlebar.rust_adblock_active_p0_blocked_click_to_pause_91a8", { p0: $privacyStats.blocked_count })
          : $t("titlebar.rust_adblock_paused_click_to_enable_5fe1")}
        aria-label={$t("titlebar.toggle_adblock_shield_6c4c")}
      >
        <span>🛡️</span>
        <span class="shield-dot"></span>
      </button>

      <button
        class="icon-btn"
        onclick={openSettingsModal}
        title={$t("titlebar.settings_shortcuts_p0_82b9", { p0: $keyboardShortcuts && shortcutLabel("settings") })}
        data-testid="open-settings" aria-label={$t("titlebar.settings_74a8")}
      >
        ⚙️
      </button>

      <button
        class="icon-btn"
        onclick={toggleZenMode}
        title={$t("titlebar.enter_zen_focus_mode_p0_d689", { p0: $keyboardShortcuts && shortcutLabel("zen") })}
        aria-label={$t("titlebar.zen_mode_826b")}
      >
        🧘
      </button>

      {#if $tabLayout === "vertical"}
        <div class="divider"></div>

        <button class="win-btn" onclick={minimizeWindow} title={$t("titlebar.minimize_d72e")} aria-label={$t("titlebar.minimize_d72e")}>
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="2" y="5.5" width="8" height="1" fill="currentColor" />
          </svg>
        </button>
        <button class="win-btn" onclick={toggleMaximizeWindow} title={$t("titlebar.maximize_fba6")} aria-label={$t("titlebar.maximize_fba6")}>
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
        <button class="win-btn close-btn" onclick={closeWindow} title={$t("titlebar.close_7d9e")} aria-label={$t("titlebar.close_7d9e")}>
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
