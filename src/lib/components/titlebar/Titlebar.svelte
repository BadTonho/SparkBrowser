<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { onMount, onDestroy, untrack } from "svelte";
  import { get } from "svelte/store";
  import MediaButton from "../media/MediaButton.svelte";
  import PipButton from "../media/PipButton.svelte";
  import ReaderModeButton from "../reader/ReaderModeButton.svelte";
  import WebStoreInstallButton from "./WebStoreInstallButton.svelte";
  import ZoomControls from "./ZoomControls.svelte";
  import TitlebarActions from "./TitlebarActions.svelte";
  import DownloadProgressIndicator from "../downloads/DownloadProgressIndicator.svelte";
  import { downloadProgressText } from "../../stores/downloadsStore";
  import { t } from "../../i18n";
  import { isNewTab } from "../../services/new-tab";
  import { openSitePermissions } from "../../stores/sitePermissionsStore";
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
  } from "../../stores/persistenceStore";
  import {
    isZenMode,
    tabLayout,
    workspaceSwitcherDisplay,
    wideAddressBar,
    showFullUrls,
    compactToolbar,
  } from "../../stores/settingsStore";
  import { formatAddressForDisplay } from "../../services/visual-preferences.js";
  import { suspiciousHost } from "../../services/suspicious-host.js";
  import { workspaces, activeWorkspaceId } from "../../stores/workspaceStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import PrivateBadge from "./PrivateBadge.svelte";
  import { defaultSearchEngine } from "../../stores/searchEngineStore";
  import WorkspaceSwitcher from "../workspaces/WorkspaceSwitcher.svelte";
  import WindowControls from "./WindowControls.svelte";
  import { splitLayout } from "../../stores/splitViewStore";
  import { titlebarMenu, openTitlebarMenu, closeTitlebarMenu } from "../../stores/titlebarMenuStore";

  let narrow = $state(typeof window !== "undefined" && window.matchMedia("(max-width: 1000px)").matches);
  const splitHorizontal = $derived($tabLayout === "horizontal" && Boolean($splitLayout));
  const responsiveCompact = $derived(splitHorizontal && narrow);
  const internalPage = $derived(isNewTab($currentUrl));
  const displayedAddress = $derived(formatAddressForDisplay($addressInput, $showFullUrls || $isAddressEditing));
  const suspicious = $derived(suspiciousHost($currentUrl, $bookmarks.map(item => item.url)));
  let showSuspiciousHost = $state(false);
  onMount(() => {
    const media = window.matchMedia("(max-width: 1000px)");
    const update = () => { narrow = media.matches; };
    media.addEventListener("change", update);
    update();
    return () => media.removeEventListener("change", update);
  });
  onDestroy(() => { void closeTitlebarMenu(); });
  $effect(() => {
    const enabled = responsiveCompact && !$isZenMode;
    const workspaceId = $activeWorkspaceId;
    untrack(() => {
      if (!enabled || get(titlebarMenu)?.workspaceId !== workspaceId) void closeTitlebarMenu();
    });
  });

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
    target.value = get(addressInput);
    target?.select();
  }

  function handleBookmarkToggle() {
    const activeTab = $tabs.find((t) => t.is_active);
    const title = activeTab?.title || $currentUrl;
    void openBookmarkEditor(title, $currentUrl, "star");
  }
</script>

{#if !$isZenMode}
  <header class="titlebar" class:split-horizontal={splitHorizontal} class:responsive-compact={responsiveCompact} class:density-compact={$compactToolbar} class:wide-address-bar={$wideAddressBar} data-tauri-drag-region={$tabLayout === "vertical" || splitHorizontal ? "" : undefined}>
    <div class="titlebar-left">
      <PrivateBadge />
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
          <span class="brand-name" data-tauri-drag-region>Spark</span>
          {#if activeWs}
            <span class="ws-badge"><AppIcon name={activeWs.icon} /> {activeWs.name}</span>
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
        {#if suspicious}<button type="button" class="suspicious-address" aria-expanded={showSuspiciousHost} title={$t("titlebar.suspicious_address", { host: suspicious.host })} aria-label={$t("titlebar.suspicious_address", { host: suspicious.host })} onclick={() => showSuspiciousHost = !showSuspiciousHost}><AppIcon name="alert" /></button>{#if showSuspiciousHost}<span role="status">{suspicious.host}</span>{/if}{/if}
        <button class="omnibox-icon" disabled={internalPage} onclick={openSitePermissions} aria-label={internalPage ? $t("newtab.internal") : $t("titlebar.site_permissions_a688")} title={internalPage ? $t("newtab.internal") : $currentUrl.startsWith("https:") ? $t("titlebar.https_connection_site_permissions_9979") : $t("titlebar.unencrypted_connection_site_permissions_7add")}>{internalPage ? "\u25C7" : $currentUrl.startsWith("https:") ? "\u{1F512}" : "\u26A0"}</button>
        <input
          class="omnibox-input"
          type="text"
          value={displayedAddress}
          oninput={(event) => addressInput.set((event.currentTarget as HTMLInputElement).value)}
          onkeydown={(event) => { if (event.key === "Escape") { addressInput.set(internalPage ? "" : $currentUrl); isAddressEditing.set(false); (event.currentTarget as HTMLInputElement).blur(); } else handleKeydown(event); }}
          onfocus={handleFocus}
          onblur={() => isAddressEditing.set(false)}
          placeholder="{$t("titlebar.search_49c2")} {$defaultSearchEngine} {$t("titlebar.or_enter_url_5dc7")}"
          spellcheck="false"
        />
        <MediaButton />
        <PipButton />
        <WebStoreInstallButton />
        <ReaderModeButton />
        <ZoomControls />
        <button
          class="star-btn"
          disabled={internalPage}
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
      {#if responsiveCompact}
        <button class="icon-btn" data-testid="open-titlebar-menu" aria-label={$t("titlebar.more")} aria-describedby="more-download-progress" title={$t("titlebar.more") + ($downloadProgressText ? " · " + $downloadProgressText : "")} aria-haspopup="menu" aria-expanded={Boolean($titlebarMenu)} onclick={event => openTitlebarMenu(event.currentTarget)} onkeydown={event => { if (event.key === "ArrowDown") { event.preventDefault(); void openTitlebarMenu(event.currentTarget); } }}>⋯<DownloadProgressIndicator /></button>
        <span id="more-download-progress" class="sr-only">{$downloadProgressText}</span>
      {:else}
        <TitlebarActions />
      {/if}
      {#if $tabLayout === "vertical" || splitHorizontal}
        <div class="divider"></div>
        <WindowControls />
      {/if}
    </div>
  </header>
{/if}

<style>
  .titlebar {
    --titlebar-height: 44px;
    --titlebar-control-width: 31px;
    --titlebar-control-height: 30px;
    --titlebar-window-control-width: 38px;
    --titlebar-actions-height: 36px;
    --titlebar-control-gap: 4px;
    --titlebar-control-font-size: 13px;
    --titlebar-input-font-size: 13px;
    --omnibox-height: 32px;
    height: var(--titlebar-height);
    flex: 0 0 var(--titlebar-height);
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

  /* A private window gets a purple wash and a bottom line: with no native title bar, this is what says so. */
  :global(html[data-private="true"]) .titlebar {
    background-image: linear-gradient(90deg, rgba(124, 58, 237, 0.26), rgba(124, 58, 237, 0.06) 55%, transparent);
    border-bottom: 2px solid rgba(167, 139, 250, 0.55);
  }
  :global(html[data-private="true"]) .brand-dot { background: #a78bfa; box-shadow: 0 0 10px #a78bfa; }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  .split-horizontal .titlebar-left { flex:0 1 auto; min-width:0; max-width:min(24vw,300px); }
  .split-horizontal .titlebar-center { min-width:380px; }
  .split-horizontal .omnibox-input { min-width:64px; }
  .responsive-compact .titlebar-left { max-width:96px; }
  .responsive-compact { gap:6px; }
  .density-compact {
    --titlebar-height: 40px;
    --titlebar-control-width: 27px;
    --titlebar-control-height: 26px;
    --titlebar-window-control-width: 34px;
    --titlebar-actions-height: 32px;
    --titlebar-control-gap: 3px;
    --titlebar-control-font-size: 12px;
    --titlebar-input-font-size: 12px;
    --omnibox-height: 28px;
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

  .titlebar.wide-address-bar .titlebar-center { max-width: none; }

  .nav-controls {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .nav-btn {
    width: var(--titlebar-control-width);
    height: var(--titlebar-control-height);
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid transparent;
    font-size: var(--titlebar-control-font-size);
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
    height: var(--omnibox-height);
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
    font-size: var(--titlebar-input-font-size);
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
    gap: var(--titlebar-control-gap);
    flex-shrink: 0;
  }

  .icon-btn {
    position: relative;
    width: var(--titlebar-control-width);
    height: var(--titlebar-control-height);
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--titlebar-control-font-size);
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

  .divider {
    width: 1px;
    height: 16px;
    background: var(--border-subtle);
    margin: 0 3px;
  }

</style>
