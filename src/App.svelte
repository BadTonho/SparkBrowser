<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import Titlebar from "./lib/components/titlebar/Titlebar.svelte";
  import Sidebar from "./lib/components/sidebar/Sidebar.svelte";
  import ViewportFrame from "./lib/components/viewport/ViewportFrame.svelte";
  import CommandBar from "./lib/components/commandbar/CommandBar.svelte";
  import HistoryDrawer from "./lib/components/history/HistoryDrawer.svelte";
  import DownloadsDrawer from "./lib/components/downloads/DownloadsDrawer.svelte";
  import SettingsModal from "./lib/components/settings/SettingsModal.svelte";
  import HorizontalTabBar from "./lib/components/tabs/HorizontalTabBar.svelte";
  import BookmarkBar from "./lib/components/bookmarks/BookmarkBar.svelte";
  import {
    tabs,
    currentUrl,
    addressInput,
    initializeTabs,
    applyTabsSnapshot,
    addTab,
    removeTab,
    selectTab,
    suspendInactiveTabs,
    toggleSidebar,
    triggerReload,
    moveActiveTabUp,
    moveActiveTabDown,
  } from "./lib/stores/browserStore";
  import { toggleCommandBar, openCommandBar } from "./lib/stores/commandBarStore";
  import {
    loadBookmarks,
    toggleHistoryDrawer,
  } from "./lib/stores/persistenceStore";
  import {
    privacyStats,
    initializePrivacyStats,
  } from "./lib/stores/privacyStore";
  import {
    initializeWorkspaces,
    activeWorkspaceId,
    activateWorkspace,
  } from "./lib/stores/workspaceStore";
  import { toggleDownloadsDrawer } from "./lib/stores/downloadsStore";
  import {
    toggleSettingsModal,
    autoHibernateMinutes,
    toggleZenMode,
    isZenMode,
    tabLayout,
  } from "./lib/stores/settingsStore";
  import {
    onContentUrlChanged,
    onTabsUpdated,
    onPrivacyStatsUpdated,
  } from "./lib/services/tauri-bridge";

  onMount(() => {
    initializeWorkspaces();
    initializeTabs();
    loadBookmarks();
    initializePrivacyStats();

    let unlistenUrl: (() => void) | undefined;
    let unlistenTabs: (() => void) | undefined;
    let unlistenPrivacy: (() => void) | undefined;

    onContentUrlChanged((newUrl) => {
      currentUrl.set(newUrl);
      addressInput.set(newUrl);
    }).then((fn) => {
      unlistenUrl = fn;
    });

    onTabsUpdated((updatedTabs) => {
      applyTabsSnapshot(updatedTabs);
    }).then((fn) => {
      unlistenTabs = fn;
    });

    onPrivacyStatsUpdated((stats) => {
      privacyStats.set(stats);
    }).then((fn) => {
      unlistenPrivacy = fn;
    });

    // Automatic Tab Hibernation check every 60s using configurable threshold
    const hibernationInterval = setInterval(() => {
      const thresholdSecs = get(autoHibernateMinutes) * 60;
      suspendInactiveTabs(thresholdSecs);
    }, 60_000);

    const handleGlobalShortcuts = (event: KeyboardEvent) => {
      const isUp = event.key === "ArrowUp" || event.key === "Up";
      const isDown = event.key === "ArrowDown" || event.key === "Down";
      const isAltOrCtrlShift = event.altKey || (event.ctrlKey && event.shiftKey);

      if (isAltOrCtrlShift && isUp) {
        event.preventDefault();
        moveActiveTabUp(get(activeWorkspaceId));
        return;
      } else if (isAltOrCtrlShift && isDown) {
        event.preventDefault();
        moveActiveTabDown(get(activeWorkspaceId));
        return;
      }

      // Alt + 1, 2, 3, 4: Switch Workspaces (Personal, Work, Study, Gaming)
      if (
        event.altKey &&
        !event.ctrlKey &&
        !event.metaKey &&
        ["1", "2", "3", "4"].includes(event.key)
      ) {
        event.preventDefault();
        const wsMap: Record<string, string> = {
          "1": "ws-personal",
          "2": "ws-work",
          "3": "ws-study",
          "4": "ws-gaming",
        };
        const targetId = wsMap[event.key];
        if (targetId) {
          activateWorkspace(targetId);
        }
        return;
      }

      // Ctrl + 1..9: Switch to Tab by index (Ctrl+9 switches to the last tab)
      if (
        (event.ctrlKey || event.metaKey) &&
        !event.altKey &&
        !event.shiftKey &&
        /^[1-9]$/.test(event.key)
      ) {
        event.preventDefault();
        const num = parseInt(event.key, 10);
        const currentTabs = get(tabs);
        const currentWs = get(activeWorkspaceId);
        const wsTabs = currentTabs.filter(
          (t) => !t.workspace_id || t.workspace_id === currentWs
        );
        if (wsTabs.length === 0) return;

        if (num === 9) {
          const lastTab = wsTabs[wsTabs.length - 1];
          if (lastTab) selectTab(lastTab.id);
        } else {
          const targetIndex = num - 1;
          if (targetIndex < wsTabs.length) {
            selectTab(wsTabs[targetIndex].id);
          }
        }
        return;
      }

      const key = event.key.toLowerCase();
      if (event.altKey && key === "z") {
        event.preventDefault();
        toggleZenMode();
      } else if ((event.ctrlKey || event.metaKey) && key === "b") {
        event.preventDefault();
        if (get(tabLayout) === "vertical") {
          toggleSidebar();
        }
      } else if ((event.ctrlKey || event.metaKey) && key === "r") {
        event.preventDefault();
        triggerReload();
      } else if ((event.ctrlKey || event.metaKey) && key === "t") {
        event.preventDefault();
        addTab("https://duckduckgo.com", get(activeWorkspaceId));
      } else if ((event.ctrlKey || event.metaKey) && key === "k") {
        event.preventDefault();
        toggleCommandBar();
      } else if ((event.ctrlKey || event.metaKey) && key === "l") {
        event.preventDefault();
        openCommandBar(get(currentUrl));
      } else if ((event.ctrlKey || event.metaKey) && key === "h") {
        event.preventDefault();
        toggleHistoryDrawer();
      } else if ((event.ctrlKey || event.metaKey) && key === "j") {
        event.preventDefault();
        toggleDownloadsDrawer();
      } else if ((event.ctrlKey || event.metaKey) && event.key === ",") {
        event.preventDefault();
        toggleSettingsModal();
      } else if ((event.ctrlKey || event.metaKey) && key === "w") {
        event.preventDefault();
        const active = get(tabs).find((t) => t.is_active);
        if (active) {
          removeTab(active.id);
        }
      }
    };

    window.addEventListener("keydown", handleGlobalShortcuts);
    return () => {
      clearInterval(hibernationInterval);
      if (unlistenUrl) unlistenUrl();
      if (unlistenTabs) unlistenTabs();
      if (unlistenPrivacy) unlistenPrivacy();
      window.removeEventListener("keydown", handleGlobalShortcuts);
    };
  });
</script>

<div class="browser-root">
  {#if $tabLayout === "horizontal" && !$isZenMode}
    <HorizontalTabBar />
  {/if}
  <Titlebar />
  {#if $tabLayout === "horizontal" && !$isZenMode}
    <BookmarkBar />
  {/if}
  <div class="browser-body">
    {#if $tabLayout === "vertical"}
      <Sidebar />
    {/if}
    <ViewportFrame />
  </div>
  <CommandBar />
  <HistoryDrawer />
  <DownloadsDrawer />
  <SettingsModal />
</div>

<style>
  .browser-root {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: transparent;
  }

  .browser-body {
    flex: 1;
    display: flex;
    width: 100%;
    overflow: hidden;
  }
</style>
