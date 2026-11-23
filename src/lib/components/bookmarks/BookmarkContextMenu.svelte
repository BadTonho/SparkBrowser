<script lang="ts">
  import { tr } from "../../i18n";
  import { errorText } from "../../i18n/errors";
  import { describeError } from "../../services/user-errors.js";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import {
    bookmarkContextMenuState,
    closeBookmarkContextMenu,
    deleteBookmarkFolder,
    openCreateBookmarkFolder,
    openBookmarkEditor,
    openBookmarkFolderEditor,
    openBookmarkFolderManager,
    tabBookmarkTarget,
  } from "../../stores/persistenceStore";
  import { addTab, selectTab } from "../../stores/browserStore";
  import { addTabsToGroup, createGroup, editingGroupId, removeTabsFromGroup, setGroupColor, setGroupMode, tabGroups, ungroup, GROUP_COLORS, type GroupColor } from "../../stores/tabGroupsStore";
  import { restoreClosedTab } from "../../stores/closedTabsStore";
  import { activeWorkspaceId } from "../../stores/workspaceStore";
  import { addTabDivider, editingDividerId, removeTabDivider } from "../../stores/tabDividersStore";
  import { onBookmarkContextMenuAction } from "../../services/tauri-bridge";
  import { moveTabToWindow, openBrowserWindow } from "../../services/bridge/windows";

  async function handleAction(action: string): Promise<void> {
    const menu = get(bookmarkContextMenuState);
    if (!menu) return;
    closeBookmarkContextMenu();

    try {
      if (action.startsWith("select-tab:") && menu.kind === "group-tabs") {
        await selectTab(action.slice("select-tab:".length));
      } else if (action === "create-group" && menu.kind === "tab") {
        createGroup(menu.targets);
      } else if (action.startsWith("add-to-group:") && menu.kind === "tab") {
        addTabsToGroup(menu.targets, action.slice("add-to-group:".length));
      } else if (action === "remove-from-group" && menu.kind === "tab") {
        removeTabsFromGroup(menu.targets);
      } else if (menu.kind === "group") {
        const group = get(tabGroups).groups.find((item) => item.id === menu.groupId);
        if (action === "rename-group") editingGroupId.set(menu.groupId);
        else if (action === "ungroup") ungroup(menu.groupId);
        else if (action === "toggle-group-mode" && group) setGroupMode(menu.groupId, group.mode === "popup" ? "collapse" : "popup");
        else if (action.startsWith("color:")) {
          const color = action.slice("color:".length) as GroupColor;
          if (GROUP_COLORS.includes(color)) setGroupColor(menu.groupId, color);
        }
      } else if (action === "new-tab" && menu.kind === "tab") {
        await addTab(undefined, get(activeWorkspaceId));
      } else if (action === "reopen-closed-tab" && menu.kind === "tab") {
        await restoreClosedTab();
      } else if (action === "bookmark-tab" && menu.kind === "tab") {
        const tab = tabBookmarkTarget(menu);
        if (tab) await openBookmarkEditor(tab.title, tab.url);
      } else if (action === "add-divider" && menu.kind === "tab") {
        addTabDivider(menu.tabId);
      } else if (action === "rename-divider" && menu.kind === "divider") {
        editingDividerId.set(menu.dividerId);
      } else if (action === "remove-divider" && menu.kind === "divider") {
        removeTabDivider(menu.dividerId);
      } else if (action.startsWith("move-window:") && menu.kind === "tab") {
        const target = action.slice("move-window:".length);
        const windowId = target === "new" ? await openBrowserWindow() : target;
        if (windowId) await moveTabToWindow(menu.tabId, windowId);
      } else if ((action === "edit-bookmark" || action === "delete-bookmark") && menu.kind === "bookmark") {
        await openBookmarkEditor(menu.bookmark.title, menu.bookmark.url, menu.bookmark.icon);
      } else if (action === "rename-folder" && menu.kind === "folder") {
        await openBookmarkFolderEditor(menu.folder);
      } else if (action === "create-folder" && menu.kind === "empty") {
        await openCreateBookmarkFolder();
      } else if (action === "manage-folders" && menu.kind === "empty") {
        await openBookmarkFolderManager();
      } else if (action === "delete-folder" && menu.kind === "folder") {
        if (window.confirm(tr("common.delete_folder", { name: menu.folder.name }))) {
          await deleteBookmarkFolder(menu.folder.id);
        }
      }
    } catch (error) {
      const message = describeError(error);
      window.alert(get(errorText)(message));
    }
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape" || !get(bookmarkContextMenuState)) return;
    event.preventDefault();
    closeBookmarkContextMenu();
  }

  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    onBookmarkContextMenuAction((action) => void handleAction(action)).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

<svelte:window onkeydown={handleWindowKeydown} />
