<script lang="ts">
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

  async function handleAction(action: string): Promise<void> {
    const menu = get(bookmarkContextMenuState);
    if (!menu) return;
    closeBookmarkContextMenu();

    try {
      if ((action === "edit-bookmark" || action === "delete-bookmark") && menu.kind === "bookmark") {
        await openBookmarkEditor(menu.bookmark.title, menu.bookmark.url, menu.bookmark.icon);
      } else if (action === "rename-folder" && menu.kind === "folder") {
        await openBookmarkFolderEditor(menu.folder);
      } else if (action === "create-folder" && menu.kind === "empty") {
        await openCreateBookmarkFolder();
      } else if (action === "manage-folders" && menu.kind === "empty") {
        await openBookmarkFolderManager();
      } else if (action === "delete-folder" && menu.kind === "folder") {
        if (window.confirm(`Delete folder “${menu.folder.name}”? Its bookmarks will be moved to the root.`)) {
          await deleteBookmarkFolder(menu.folder.id);
        }
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      window.alert(message);
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
