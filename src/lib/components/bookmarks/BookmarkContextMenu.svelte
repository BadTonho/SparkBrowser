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
  } from "../../stores/persistenceStore";
  import { onBookmarkContextMenuAction } from "../../services/tauri-bridge";
  import { moveTabToWindow, openBrowserWindow } from "../../services/bridge/windows";

  async function handleAction(action: string): Promise<void> {
    const menu = get(bookmarkContextMenuState);
    if (!menu) return;
    closeBookmarkContextMenu();

    try {
      if (action.startsWith("move-window:") && menu.kind === "tab") {
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
