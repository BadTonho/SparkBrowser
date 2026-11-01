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
  } from "./lib/stores/workspaceStore";
  import { toggleDownloadsDrawer } from "./lib/stores/downloadsStore";
  import {
    toggleSettingsModal,
    autoHibernateMinutes,
    toggleZenMode,
    isZenMode,
    tabLayout,
    showBookmarksBar,
  } from "./lib/stores/settingsStore";
  import {
    SHORTCUT_DEFINITIONS,
    getShortcutBinding,
    initializeKeyboardShortcuts,
    isWorkspacePickerHoldKey,
    keyboardShortcuts,
    matchesShortcut,
    recordingShortcutId,
  } from "./lib/stores/shortcutsStore";
  import {
    onContentUrlChanged,
    onTabsUpdated,
    onPrivacyStatsUpdated,
    onBrowserShortcutAction,
  } from "./lib/services/tauri-bridge";
  import WorkspaceSwitcherOverlay from "./lib/components/workspaces/WorkspaceSwitcherOverlay.svelte";
  import {
    cancelWorkspaceAltHoldTimer,
    dismissWorkspacePicker,
    handleWorkspaceShortcutEvent,
    initializeWorkspaceShortcutBridge,
    releaseWorkspaceAlt,
    startWorkspaceAltHold,
    workspaceSwitcherOverlay,
  } from "./lib/stores/workspaceSwitcherStore";

  onMount(() => {
    initializeWorkspaces();
    initializeTabs();
    loadBookmarks();
    initializePrivacyStats();
    initializeKeyboardShortcuts();

    let unlistenUrl: (() => void) | undefined;
    let unlistenTabs: (() => void) | undefined;
    let unlistenPrivacy: (() => void) | undefined;
    let unlistenBrowserShortcut: (() => void) | undefined;
    const stopWorkspaceShortcutBridge = initializeWorkspaceShortcutBridge();

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

    onBrowserShortcutAction((action, key) => {
      dispatchShortcut(action, key);
    }).then((fn) => {
      unlistenBrowserShortcut = fn;
    });

    // Automatic Tab Hibernation check every 60s using configurable threshold
    const hibernationInterval = setInterval(() => {
      const thresholdSecs = get(autoHibernateMinutes) * 60;
      suspendInactiveTabs(thresholdSecs);
    }, 60_000);

    const handleGlobalShortcuts = (event: KeyboardEvent) => {
      if (get(recordingShortcutId)) return;
      const openWorkspaceOverlay = get(workspaceSwitcherOverlay);

      if (event.key === "Escape" && openWorkspaceOverlay) {
        event.preventDefault();
        event.stopImmediatePropagation();
        if (openWorkspaceOverlay.kind === "picker") dismissWorkspacePicker();
        return;
      }

      if (isWorkspacePickerHoldKey(event)) {
        if (!event.repeat) {
          event.preventDefault();
          startWorkspaceAltHold();
        }
        return;
      }

      const config = get(keyboardShortcuts);
      for (const definition of SHORTCUT_DEFINITIONS) {
        const binding = getShortcutBinding(config, definition.id);
        if (!binding || definition.hold || !matchesShortcut(event, binding, definition.sequence)) continue;
        if (openWorkspaceOverlay && definition.id !== "switch-workspaces") return;
        event.preventDefault();
        if (definition.id !== "switch-workspaces") cancelWorkspaceAltHoldTimer();
        if (!event.repeat) dispatchShortcut(definition.id, event.key);
        return;
      }

      if (!openWorkspaceOverlay && (event.ctrlKey || event.altKey || event.shiftKey || event.metaKey)) {
        cancelWorkspaceAltHoldTimer();
      }
    };

    const handleGlobalKeyup = (event: KeyboardEvent) => {
      if (isWorkspacePickerHoldKey(event)) {
        releaseWorkspaceAlt();
      }
    };

    window.addEventListener("keydown", handleGlobalShortcuts, true);
    window.addEventListener("keyup", handleGlobalKeyup, true);
    return () => {
      clearInterval(hibernationInterval);
      if (unlistenUrl) unlistenUrl();
      if (unlistenTabs) unlistenTabs();
      if (unlistenPrivacy) unlistenPrivacy();
      if (unlistenBrowserShortcut) unlistenBrowserShortcut();
      stopWorkspaceShortcutBridge();
      window.removeEventListener("keydown", handleGlobalShortcuts, true);
      window.removeEventListener("keyup", handleGlobalKeyup, true);
    };
  });

  function dispatchShortcut(action: string, key?: string): void {
    switch (action) {
      case "switch-tabs": {
        const num = Number(key);
        const currentWs = get(activeWorkspaceId);
        const wsTabs = get(tabs).filter((tab) => !tab.workspace_id || tab.workspace_id === currentWs);
        if (wsTabs.length === 0) return;
        const target = num === 9 ? wsTabs.at(-1) : wsTabs[num - 1];
        if (target) selectTab(target.id);
        break;
      }
      case "workspace-picker":
        startWorkspaceAltHold();
        break;
      case "switch-workspaces":
        if (key && /^[1-9]$/.test(key)) {
          void handleWorkspaceShortcutEvent({ type: "select-index", index: Number(key) });
        }
        break;
      case "move-tab-up":
      case "move-tab-up-ctrl-shift": moveActiveTabUp(get(activeWorkspaceId)); break;
      case "move-tab-down":
      case "move-tab-down-ctrl-shift": moveActiveTabDown(get(activeWorkspaceId)); break;
      case "new-tab": addTab("https://duckduckgo.com", get(activeWorkspaceId)); break;
      case "command-bar": toggleCommandBar(); break;
      case "edit-url": openCommandBar(get(currentUrl)); break;
      case "sidebar": if (get(tabLayout) === "vertical") toggleSidebar(); break;
      case "zen": toggleZenMode(); break;
      case "history": toggleHistoryDrawer(); break;
      case "downloads": toggleDownloadsDrawer(); break;
      case "settings": toggleSettingsModal(); break;
      case "close-tab": {
        const active = get(tabs).find((tab) => tab.is_active);
        if (active) removeTab(active.id);
        break;
      }
      case "reload-tab": triggerReload(); break;
    }
  }
</script>

<div class="browser-root">
  {#if $tabLayout === "horizontal" && !$isZenMode}
    <HorizontalTabBar />
  {/if}
  <Titlebar />
  {#if $tabLayout === "horizontal" && !$isZenMode && $showBookmarksBar}
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
  <WorkspaceSwitcherOverlay />
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
