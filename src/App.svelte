<script lang="ts">
  import RuntimeNotice from "./lib/components/status/RuntimeNotice.svelte";
  import { initializeRuntime } from "./lib/stores/webviewRuntimeStore";
  import { initializeNewTab } from "./lib/services/new-tab";
  import { focusAddressBar } from "./lib/services/bridge/navigation";
  import { openBrowserWindow } from "./lib/services/bridge/windows";
  import ShutdownDialog from "./lib/components/shutdown/ShutdownDialog.svelte";
  import { initializeShutdown, shutdownState } from "./lib/stores/shutdownStore";
  import { initializeTabDrag } from "./lib/stores/tabDragStore";
  import { splitLayout } from "./lib/stores/splitViewStore";
  import { focusedTabs } from "./lib/services/split-tab-groups.js";
  import { initializeMedia } from "./lib/stores/mediaLifecycle";
  import { initializeWorkspaceHibernation } from "./lib/stores/workspaceHibernation";
  import { initializeZoom, changeZoom } from "./lib/stores/zoomStore";
  import RecoveryNotice from "./lib/components/status/RecoveryNotice.svelte";
  import { initializeRecovery } from "./lib/stores/recoveryStore";
  import ClosedTabsNotice from "./lib/components/status/ClosedTabsNotice.svelte";
  import { initializeClosedTabEvents, restoreClosedTab } from "./lib/stores/closedTabsStore";
  import BackupNotice from "./lib/components/status/BackupNotice.svelte";
  import { initializeBackupStatus } from "./lib/stores/backupStore";
  import { initializePrivateWindow } from "./lib/stores/privateWindowStore";
  import { initializeBrowserSession } from "./lib/stores/browser/lifecycle";
  import { onMount } from "svelte";
  import { initializeUpdates } from "./lib/stores/updatesStore";
  import UpdateNotice from "./lib/components/status/UpdateNotice.svelte";
  import PermissionPrompt from "./lib/components/permissions/PermissionPrompt.svelte";
  import PermissionPopoverHost from "./lib/components/permissions/PermissionPopoverHost.svelte";
  import { initializePermissionEvents } from "./lib/stores/sitePermissionsStore";
  import PrivacyPanel from "./lib/components/privacy/PrivacyPanel.svelte";
  import MediaPanel from "./lib/components/media/MediaPanel.svelte";
  import ZoomPanel from "./lib/components/titlebar/ZoomPanel.svelte";
  import MediaNotice from "./lib/components/media/MediaNotice.svelte";
  import BrowserErrorBanner from "./lib/components/status/BrowserErrorBanner.svelte";
  import ToastNotification from "./lib/components/notifications/ToastNotification.svelte";
  import ErrorReportToast from "./lib/components/status/ErrorReportToast.svelte";
  import { slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { get } from "svelte/store";
  import Titlebar from "./lib/components/titlebar/Titlebar.svelte";
  import TitlebarMenu from "./lib/components/titlebar/TitlebarMenu.svelte";
  import QrCodePanel from "./lib/components/titlebar/QrCodePanel.svelte";
  import Sidebar from "./lib/components/sidebar/Sidebar.svelte";
  import ViewportFrame from "./lib/components/viewport/ViewportFrame.svelte";
  import CommandBar from "./lib/components/commandbar/CommandBar.svelte";
  import TabSelectionActions from "./lib/components/tabs/TabSelectionActions.svelte";
  import { refreshExtensions, isExtensionsModalOpen, activeExtensionPopup } from "./lib/stores/extensionsStore";
  import { isHistoryDrawerOpen, bookmarkEditorState } from "./lib/stores/persistenceStore";
  import { isDownloadsDrawerOpen } from "./lib/stores/downloadsStore";
  import { isSettingsModalOpen, settingsTabSession, syncSettingsTabSession } from "./lib/stores/settingsStore";
  import { onSettingsTabLoaded } from "./lib/services/bridge/settings-overlay";
  import { isSettingsQuickOpen } from "./lib/stores/settingsQuickPopoverStore";
  import { isToolsOpen } from "./lib/stores/toolsStore";
  import { isSitePermissionsOpen } from "./lib/stores/sitePermissionsStore";
  import { isReaderOpen } from "./lib/stores/readerStore";
  import HorizontalTabBar from "./lib/components/tabs/HorizontalTabBar.svelte";
  import BookmarkBar from "./lib/components/bookmarks/BookmarkBar.svelte";
  import BookmarkContextMenu from "./lib/components/bookmarks/BookmarkContextMenu.svelte";
  import {
    tabs,
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
  import { toggleCommandBar } from "./lib/stores/commandBarStore";
  import {
    bookmarks,
    bookmarkFolders,
    loadBookmarks,
    toggleHistoryDrawer,
  } from "./lib/stores/persistenceStore";
  import {
    applyPrivacyStats,
    initializePrivacyStats,
  } from "./lib/stores/privacyStore";
  import {
    activeWorkspaceId,
  } from "./lib/stores/workspaceStore";
  import { toggleDownloadsDrawer, initializeDownloadEvents } from "./lib/stores/downloadsStore";
  import { toggleVideoPip } from "./lib/stores/pipStore";
  import {
    toggleSettingsModal,
    autoHibernateMode,
    residentTabLimit,
    setResidentTabLimit,
    syncForcedDarkPages,
    hibernationExemptions,
    setHibernationExemptions,
    askDownloadDestination,
    setAskDownloadDestination,
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
    onPrivacyStatsUpdated,
    onBrowserShortcutAction,
    openPrivateWindow,
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

  // Load hidden surfaces independently on their first opening.
  let Lazy = $state.raw<Record<string, any>>({});
  const surfaceLoaders: Record<string, () => Promise<{default: any}>> = {
        HistoryDrawer: () => import("./lib/components/history/HistoryDrawer.svelte"),
        DownloadsDrawer: () => import("./lib/components/downloads/DownloadsDrawer.svelte"),
        SettingsModal: () => import("./lib/components/settings/SettingsModal.svelte"),
        SettingsQuickMenu: () => import("./lib/components/settings/SettingsQuickMenu.svelte"),
        ToolsPanel: () => import("./lib/components/tools/ToolsPanel.svelte"),
        SitePermissionsPanel: () => import("./lib/components/permissions/SitePermissionsPanel.svelte"),
        BookmarkEditor: () => import("./lib/components/bookmarks/BookmarkEditor.svelte"),
        ExtensionsModal: () => import("./lib/components/extensions/ExtensionsModal.svelte"),
        ExtensionActionPopover: () => import("./lib/components/extensions/ExtensionActionPopover.svelte"),
        ReaderOverlay: () => import("./lib/components/reader/ReaderOverlay.svelte"),
  };
  const pendingSurfaces = new Set<string>();
  async function loadSurface(name: string) {
    if (Lazy[name] || pendingSurfaces.has(name)) return;
    pendingSurfaces.add(name);
    try {
      const component = (await surfaceLoaders[name]()).default;
      Lazy = { ...Lazy, [name]: component };
    } catch (error) {
      console.error(`Could not load ${name}`, error);
    } finally {
      pendingSurfaces.delete(name);
    }
  }
  $effect(() => {
    if ($isHistoryDrawerOpen) void loadSurface("HistoryDrawer");
    if ($isDownloadsDrawerOpen) void loadSurface("DownloadsDrawer");
    if ($isSettingsModalOpen || $settingsTabSession) void loadSurface("SettingsModal");
    if ($isSettingsQuickOpen) void loadSurface("SettingsQuickMenu");
    if ($isToolsOpen) void loadSurface("ToolsPanel");
    if ($isSitePermissionsOpen) void loadSurface("SitePermissionsPanel");
    if ($bookmarkEditorState) void loadSurface("BookmarkEditor");
    if ($isExtensionsModalOpen) void loadSurface("ExtensionsModal");
    if ($activeExtensionPopup) void loadSurface("ExtensionActionPopover");
    if ($isReaderOpen) void loadSurface("ReaderOverlay");
  });

  $effect(() => {
    const tab = $tabs.find(item => item.url === "spark://settings" && (item.is_active || item.is_visible));
    syncSettingsTabSession(tab?.id ?? null);
  });

  onMount(() => {
    const stopTabDrag = initializeTabDrag();
    const stopMedia = initializeMedia();
    const stopZoom = initializeZoom();
    const stopNewTab = initializeNewTab();
    const stopShutdown = initializeShutdown();
    const stopRuntime = initializeRuntime();
    void initializeRecovery();
    const stopBackup = initializeBackupStatus();
    const stopPrivateWindow = initializePrivateWindow();
    const stopClosedTabs = initializeClosedTabEvents();
    const stopUpdates = initializeUpdates();
    const stopPermissionEvents = initializePermissionEvents();
    const stopDownloadEvents = initializeDownloadEvents();
    void refreshExtensions();
    const motionPreference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotionPreference = () => {
      prefersReducedMotion = motionPreference.matches;
    };
    motionPreference.addEventListener("change", updateMotionPreference);


    let unlistenPrivacy: (() => void) | undefined;
    let unlistenBrowserShortcut: (() => void) | undefined;
    let unlistenSettingsTab: (() => void) | undefined;
    const stopWorkspaceShortcutBridge = initializeWorkspaceShortcutBridge();

    let disposed = false;
    const session = initializeBrowserSession();
    void onSettingsTabLoaded(tabId => {
      if (get(settingsTabSession)?.tabId === tabId) syncSettingsTabSession(tabId, true);
    }).then(stop => { if (disposed) stop(); else unlistenSettingsTab = stop; });

    const privacyListener = onPrivacyStatsUpdated((stats) => {
      applyPrivacyStats(stats);
    }).then((fn) => {
      if (disposed) fn(); else unlistenPrivacy = fn;
    });

    const shortcutListener = onBrowserShortcutAction((action, key) => {
      dispatchShortcut(action, key);
    }).then((fn) => {
      if (disposed) fn(); else unlistenBrowserShortcut = fn;
    });

    void Promise.all([session.ready, privacyListener, shortcutListener]).then(async () => {
      if (disposed) return;
      void syncForcedDarkPages();
      await setHibernationExemptions(get(hibernationExemptions));
      await setAskDownloadDestination(get(askDownloadDestination));
      await Promise.all([setResidentTabLimit(get(residentTabLimit)), loadBookmarks(), initializePrivacyStats(), initializeKeyboardShortcuts()]);
    });

    const stopWorkspaceHibernation = initializeWorkspaceHibernation();

    // Apply the selected automatic hibernation policy once per minute.
    const hibernationInterval = setInterval(() => {
      const mode = get(autoHibernateMode);
      void suspendInactiveTabs(mode);
    }, 60_000);

    const handleGlobalShortcuts = (event: KeyboardEvent) => {
      if (get(shutdownState).phase !== "idle") return;
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
      if (get(shutdownState).phase !== "idle") return;
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
      stopTabDrag();
      stopMedia();
      stopZoom();
      stopNewTab();
      stopShutdown();
      stopRuntime();
      stopPermissionEvents();
      stopUpdates();
      stopClosedTabs();
      stopBackup();
      stopPrivateWindow();
      stopDownloadEvents();
      clearInterval(hibernationInterval);
      stopWorkspaceHibernation();
      disposed = true;
      session.stop();
      if (unlistenPrivacy) unlistenPrivacy();
      if (unlistenBrowserShortcut) unlistenBrowserShortcut();
      if (unlistenSettingsTab) unlistenSettingsTab();
      stopWorkspaceShortcutBridge();
      motionPreference.removeEventListener("change", updateMotionPreference);
      window.removeEventListener("keydown", handleGlobalShortcuts, true);
      window.removeEventListener("keyup", handleGlobalKeyup, true);
      window.removeEventListener("mousedown", handleGlobalMousedown, true);
      window.removeEventListener("auxclick", handleGlobalAuxClick, true);
    };
  });

  function dispatchShortcut(action: string, key?: string): void {
    if (get(shutdownState).phase !== "idle") return;
    switch (action) {
      case "zoom-in": void changeZoom(1); break;
      case "zoom-out": void changeZoom(-1); break;
      case "zoom-reset": void changeZoom(0); break;
      case "switch-tabs": {
        const num = Number(key);
        const currentWs = get(activeWorkspaceId);
        const wsTabs = focusedTabs(get(tabs), get(splitLayout), currentWs);
        if (wsTabs.length === 0) return;
        const target = num === 9 ? wsTabs.at(-1) : wsTabs[num - 1];
        if (target) selectTab(target.id);
        break;
      }
      case "next-tab":
      case "previous-tab": {
        const list = focusedTabs(get(tabs), get(splitLayout), get(activeWorkspaceId));
        const index = list.findIndex(tab => tab.is_active);
        const target = list[(index + (action === "next-tab" ? 1 : -1) + list.length) % list.length];
        if (target) void selectTab(target.id);
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
      case "new-window": openBrowserWindow().catch((error) => console.warn(error)); break;
      case "private-window": openPrivateWindow().catch((error) => console.warn(error)); break;
      case "navigate-back": void triggerBack(); break;
      case "navigate-forward": void triggerForward(); break;
      case "command-bar": toggleCommandBar(); break;
      case "edit-url":
      case "focus-address":
        void focusAddressBar().then(() => document.querySelector<HTMLInputElement>(".omnibox-input")?.focus());
        break;
      case "sidebar": if (get(tabLayout) === "vertical") toggleSidebar(); break;
      case "zen": toggleZenMode(); break;
      case "history": toggleHistoryDrawer(); break;
      case "downloads": toggleDownloadsDrawer(); break;
      case "settings": toggleSettingsModal(); break;
      case "close-tab": {
        const active = get(tabs).find((tab) => tab.is_active);
        if (active) void removeTab(active.id, true);
        break;
      }
      case "reopen-tab": void restoreClosedTab(); break;
      case "reload-tab": triggerReload(); break;
      case "toggle-pip": void toggleVideoPip(); break;
    }
  }
</script>

<div class="browser-root">
  <BrowserErrorBanner />
  <ToastNotification />
  <ErrorReportToast />
  <MediaNotice />
  <MediaPanel />
  <ZoomPanel />
  <PrivacyPanel />
  <div class="browser-chrome">
    {#if $tabLayout === "horizontal" && !$isZenMode}
      <div class="horizontal-tabs-host" class:below-address={Boolean($splitLayout)}>
        <HorizontalTabBar />
      </div>
    {/if}
    <Titlebar />
    <UpdateNotice />
  <RuntimeNotice />
    <BackupNotice />
    <RecoveryNotice />
    <ClosedTabsNotice />
    {#if $tabLayout === "horizontal" && !$isZenMode && $showBookmarksBar && ($bookmarks.length > 0 || $bookmarkFolders.length > 0)}
      <div
        class="bookmark-bar-transition"
        transition:slide={{ duration: prefersReducedMotion ? 0 : 180, easing: cubicOut }}
      >
        <BookmarkBar />
      </div>
    {/if}
  </div>
  <div class="browser-body">
    {#if $tabLayout === "vertical"}
      <Sidebar />
    {/if}
    <ViewportFrame />
  </div>
  <ShutdownDialog />
  <TitlebarMenu />
  <QrCodePanel />
  <CommandBar />
  <TabSelectionActions />
  {#if Lazy.HistoryDrawer}<Lazy.HistoryDrawer />{/if}
  {#if Lazy.DownloadsDrawer}<Lazy.DownloadsDrawer />{/if}
  {#if Lazy.SettingsModal}<Lazy.SettingsModal />{/if}
  {#if Lazy.SettingsQuickMenu}<Lazy.SettingsQuickMenu />{/if}
  {#if Lazy.ToolsPanel}<Lazy.ToolsPanel />{/if}
  {#if Lazy.SitePermissionsPanel}<Lazy.SitePermissionsPanel />{/if}
  <PermissionPrompt />
  <PermissionPopoverHost />
  {#if Lazy.BookmarkEditor}<Lazy.BookmarkEditor />{/if}
  <BookmarkContextMenu />
  <WorkspaceSwitcherOverlay />
  {#if Lazy.ExtensionsModal}<Lazy.ExtensionsModal />{/if}
  {#if Lazy.ExtensionActionPopover}<Lazy.ExtensionActionPopover />{/if}
  {#if Lazy.ReaderOverlay}<Lazy.ReaderOverlay />{/if}
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

  .browser-chrome { display:flex; flex-direction:column; flex:0 0 auto; min-width:0; }
  .horizontal-tabs-host { order:-1; flex:0 0 38px; }
  .horizontal-tabs-host.below-address { order:1; }

  .bookmark-bar-transition {
    flex: 0 0 auto;
    width: 100%;
    overflow: hidden;
  }
</style>
