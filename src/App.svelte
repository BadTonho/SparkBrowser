<script lang="ts">
  import BackupNotice from "./lib/components/status/BackupNotice.svelte";
  import { initializeBackupStatus } from "./lib/stores/backupStore";
  import { onMount } from "svelte";
  import { initializeUpdates } from "./lib/stores/updatesStore";
  import UpdateNotice from "./lib/components/status/UpdateNotice.svelte";
  import SitePermissionsPanel from "./lib/components/permissions/SitePermissionsPanel.svelte";
  import PermissionPrompt from "./lib/components/permissions/PermissionPrompt.svelte";
  import { initializePermissionEvents } from "./lib/stores/sitePermissionsStore";
  import ToolsPanel from "./lib/components/tools/ToolsPanel.svelte";
  import BrowserErrorBanner from "./lib/components/status/BrowserErrorBanner.svelte";
  import { slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
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
  import BookmarkEditor from "./lib/components/bookmarks/BookmarkEditor.svelte";
  import BookmarkContextMenu from "./lib/components/bookmarks/BookmarkContextMenu.svelte";
  import {
    tabs,
    applyNavigationPatch,
    currentUrl,
    initializeTabs,
    applyTabsSnapshot,
    addTab,
    removeTab,
    selectTab,
    suspendInactiveTabs,
    toggleSidebar,
    triggerBack,
    triggerForward,
    triggerReload,
    moveActiveTabUp,
    moveActiveTabDown,
  } from "./lib/stores/browserStore";
  import { toggleCommandBar, openCommandBar } from "./lib/stores/commandBarStore";
  import {
    bookmarks,
    bookmarkFolders,
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
  import { toggleDownloadsDrawer, initializeDownloadEvents } from "./lib/stores/downloadsStore";
  import {
    toggleSettingsModal,
    autoHibernateMode,
    residentTabLimit,
    setResidentTabLimit,
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
    matchesMouseShortcut,
    recordingShortcutId,
  } from "./lib/stores/shortcutsStore";
  import {
    onNavigationUpdated,
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

  let prefersReducedMotion = $state(
    typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches
  );

  onMount(() => {
    initializeBackupStatus();
    const stopUpdates = initializeUpdates();
    const stopPermissionEvents = initializePermissionEvents();
    const stopDownloadEvents = initializeDownloadEvents();
    const motionPreference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotionPreference = () => {
      prefersReducedMotion = motionPreference.matches;
    };
    motionPreference.addEventListener("change", updateMotionPreference);


    let unlistenNavigation: (() => void) | undefined;
    let unlistenTabs: (() => void) | undefined;
    let unlistenPrivacy: (() => void) | undefined;
    let unlistenBrowserShortcut: (() => void) | undefined;
    const stopWorkspaceShortcutBridge = initializeWorkspaceShortcutBridge();

    let disposed = false;
    const navigationListener = onNavigationUpdated(applyNavigationPatch).then(fn => {
      if (disposed) fn(); else unlistenNavigation = fn;
    });

    const tabsListener = onTabsUpdated((updatedTabs) => {
      applyTabsSnapshot(updatedTabs);
    }).then((fn) => {
      if (disposed) fn(); else unlistenTabs = fn;
    });

    const privacyListener = onPrivacyStatsUpdated((stats) => {
      privacyStats.set(stats);
    }).then((fn) => {
      if (disposed) fn(); else unlistenPrivacy = fn;
    });

    const shortcutListener = onBrowserShortcutAction((action, key) => {
      dispatchShortcut(action, key);
    }).then((fn) => {
      if (disposed) fn(); else unlistenBrowserShortcut = fn;
    });

    void Promise.all([navigationListener, tabsListener, privacyListener, shortcutListener]).then(async () => {
      if (disposed) return;
      await initializeWorkspaces();
      if (disposed) return;
      await initializeTabs();
      if (disposed) return;
      await Promise.all([setResidentTabLimit(get(residentTabLimit)), loadBookmarks(), initializePrivacyStats(), initializeKeyboardShortcuts()]);
    });

    // Apply the selected automatic hibernation policy once per minute.
    const hibernationInterval = setInterval(() => {
      const mode = get(autoHibernateMode);
      void suspendInactiveTabs(mode);
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

    const handleGlobalMousedown = (event: MouseEvent) => {
      if (event.button !== 3 && event.button !== 4) return;
      event.preventDefault();

      if (get(recordingShortcutId)) {
        // Let Settings capture the side-button binding while preventing native navigation.
        return;
      }

      const config = get(keyboardShortcuts);
      for (const definition of SHORTCUT_DEFINITIONS) {
        const binding = getShortcutBinding(config, definition.id);
        if (!binding || !matchesMouseShortcut(event, binding)) continue;
        event.stopImmediatePropagation();
        dispatchShortcut(definition.id);
        return;
      }
    };

    const handleGlobalAuxClick = (event: MouseEvent) => {
      if (event.button === 3 || event.button === 4) {
        event.preventDefault();
        event.stopImmediatePropagation();
      }
    };

    window.addEventListener("keydown", handleGlobalShortcuts, true);
    window.addEventListener("keyup", handleGlobalKeyup, true);
    window.addEventListener("mousedown", handleGlobalMousedown, true);
    window.addEventListener("auxclick", handleGlobalAuxClick, true);
    return () => {
      stopPermissionEvents();
      stopUpdates();
      stopDownloadEvents();
      clearInterval(hibernationInterval);
      disposed = true;
      if (unlistenNavigation) unlistenNavigation();
      if (unlistenTabs) unlistenTabs();
      if (unlistenPrivacy) unlistenPrivacy();
      if (unlistenBrowserShortcut) unlistenBrowserShortcut();
      stopWorkspaceShortcutBridge();
      motionPreference.removeEventListener("change", updateMotionPreference);
      window.removeEventListener("keydown", handleGlobalShortcuts, true);
      window.removeEventListener("keyup", handleGlobalKeyup, true);
      window.removeEventListener("mousedown", handleGlobalMousedown, true);
      window.removeEventListener("auxclick", handleGlobalAuxClick, true);
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
      case "new-tab": addTab(undefined, get(activeWorkspaceId)); break;
      case "navigate-back": void triggerBack(); break;
      case "navigate-forward": void triggerForward(); break;
      case "command-bar": toggleCommandBar(); break;
      case "edit-url": openCommandBar(get(currentUrl)); break;
      case "focus-address":
        document.querySelector<HTMLInputElement>(".omnibox-input")?.focus();
        break;
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
  <BrowserErrorBanner />
  {#if $tabLayout === "horizontal" && !$isZenMode}
    <HorizontalTabBar />
  {/if}
  <Titlebar />
  <UpdateNotice />
  <BackupNotice />
  {#if $tabLayout === "horizontal" && !$isZenMode && $showBookmarksBar && ($bookmarks.length > 0 || $bookmarkFolders.length > 0)}
    <div
      class="bookmark-bar-transition"
      transition:slide={{ duration: prefersReducedMotion ? 0 : 180, easing: cubicOut }}
    >
      <BookmarkBar />
    </div>
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
  <ToolsPanel />
  <SitePermissionsPanel />
  <PermissionPrompt />
  <BookmarkEditor />
  <BookmarkContextMenu />
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

  .bookmark-bar-transition {
    flex: 0 0 auto;
    width: 100%;
    overflow: hidden;
  }
</style>
