<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { t } from "../../i18n";
  import { onDestroy } from "svelte";
  import {
    bookmarksByFolder,
    bookmarkFolders,
    showBookmarkContextMenu,
    openBookmarkFolderManager,
  } from "../../stores/persistenceStore";
  import { currentUrl, submitNavigation } from "../../stores/browserStore";

  let openFolderId = $state<number | null>(null);
  let menuPosition = $state({ left: 0, top: 0 });

  function toggleFolder(event: MouseEvent, folderId: number): void {
    if (openFolderId === folderId) {
      openFolderId = null;
      return;
    }
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    menuPosition = { left: Math.min(rect.left, window.innerWidth - 280), top: rect.bottom + 4 };
    openFolderId = folderId;
  }

  function closeOnOutside(event: PointerEvent): void {
    if (!(event.target instanceof Element) || !event.target.closest(".bookmark-folder-wrap, .bookmark-folder-menu")) {
      openFolderId = null;
    }
  }

  function navigate(url: string): void {
    openFolderId = null;
    void submitNavigation(url);
  }

  function showEmptyContextMenu(event: MouseEvent): void {
    if (event.target instanceof Element && event.target.closest(".bookmark-item, .folder-menu-item")) return;
    showBookmarkContextMenu(event, { kind: "empty" });
  }

  const outsideListener = (event: PointerEvent) => closeOnOutside(event);
  $effect(() => {
    if (openFolderId !== null) window.addEventListener("pointerdown", outsideListener);
    else window.removeEventListener("pointerdown", outsideListener);
  });
  onDestroy(() => window.removeEventListener("pointerdown", outsideListener));

  let rootBookmarks = $derived(($bookmarksByFolder.get(null) ?? []));
  let openFolder = $derived($bookmarkFolders.find((folder) => folder.id === openFolderId));
  let openFolderBookmarks = $derived(($bookmarksByFolder.get(openFolderId) ?? []));
</script>

<nav class="bookmark-bar" aria-label={$t("bookmarks.bookmarks_9631")} oncontextmenu={showEmptyContextMenu}>
  <div class="bookmark-list">
    {#each rootBookmarks as item (item.id)}
      <button
        class="bookmark-item"
        class:active={$currentUrl === item.url}
        onclick={() => navigate(item.url)}
        oncontextmenu={(event) => showBookmarkContextMenu(event, { kind: "bookmark", bookmark: item })}
        title={item.url}
        aria-label="{$t("bookmarks.open_bookmark_9217")} {item.title}"
      >
        <span class="bookmark-icon" aria-hidden="true"><AppIcon name={item.icon} /></span>
        <span class="bookmark-title">{item.title}</span>
      </button>
    {/each}
    {#each $bookmarkFolders as folder (folder.id)}
      <div class="bookmark-folder-wrap">
        <button
          class="bookmark-item folder-button"
          class:folder-open={openFolderId === folder.id}
          onclick={(event) => toggleFolder(event, folder.id)}
          oncontextmenu={(event) => showBookmarkContextMenu(event, { kind: "folder", folder })}
          aria-expanded={openFolderId === folder.id}
          aria-label="{$t("bookmarks.open_folder_ae74")} {folder.name}"
        >
          <span aria-hidden="true"><AppIcon name="folder" /></span>
          <span class="bookmark-title">{folder.name}</span>
          <span class="folder-count">{($bookmarksByFolder.get(folder.id) ?? []).length}</span>
        </button>
      </div>
    {/each}
  </div>
</nav>

{#if openFolder}
  <div
    class="bookmark-folder-menu"
    style:left="{Math.max(8, menuPosition.left)}px"
    style:top="{menuPosition.top}px"
    role="menu"
    aria-label="{openFolder.name} {$t("bookmarks.bookmarks_37d1")}"
  >
    <div class="folder-menu-heading"><AppIcon name="folder" /> {openFolder.name}</div>
    {#if openFolderBookmarks.length === 0}
      <span class="bookmark-empty">{$t("bookmarks.this_folder_is_empty_3043")}</span>
    {:else}
      {#each openFolderBookmarks as item (item.id)}
        <button
          class="folder-menu-item"
          role="menuitem"
          onclick={() => navigate(item.url)}
          oncontextmenu={(event) => showBookmarkContextMenu(event, { kind: "bookmark", bookmark: item })}
          title={item.url}
        >
          <span aria-hidden="true"><AppIcon name={item.icon} /></span><span>{item.title}</span>
        </button>
      {/each}
    {/if}
    <button class="folder-menu-manage" onclick={() => { openFolderId = null; void openBookmarkFolderManager(); }}>{$t("bookmarks.manage_folders_3642")}</button>
  </div>
{/if}

<style>
  .bookmark-bar {
    width: 100%;
    height: 32px;
    min-height: 32px;
    display: flex;
    align-items: center;
    padding: 0 10px;
    overflow: hidden;
    background: var(--bg-glass-primary);
    border-bottom: 1px solid var(--border-subtle);
  }

  .bookmark-list {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
  }
  .bookmark-list::-webkit-scrollbar { display: none; }

  .bookmark-item {
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: 220px;
    min-width: 0;
    padding: 4px 8px;
    flex: 0 0 auto;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 11px;
    transition: background 0.16s ease, color 0.16s ease;
  }
  .bookmark-item:hover, .folder-button.folder-open { background: var(--bg-glass-hover); color: var(--text-primary); }
  .bookmark-item.active { color: var(--accent-primary); background: color-mix(in srgb, var(--accent-primary) 14%, transparent); }
  .bookmark-icon { flex: 0 0 auto; }
  .bookmark-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .folder-count { color: var(--text-muted); font-size: 9px; }
  .bookmark-empty { padding: 4px 8px; color: var(--text-muted); font-size: 11px; }
  .bookmark-folder-menu {
    position: fixed;
    z-index: 1300;
    width: min(272px, calc(100vw - 16px));
    max-height: min(360px, calc(100vh - 16px));
    overflow: auto;
    padding: 6px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--bg-glass-primary);
    color: var(--text-primary);
    box-shadow: 0 12px 36px rgba(0, 0, 0, .38);
    backdrop-filter: blur(var(--overlay-backdrop-blur, 16px));
  }
  .folder-menu-heading { padding: 7px 8px; color: var(--text-muted); font-size: 10px; font-weight: 700; }
  .folder-menu-item, .folder-menu-manage { width: 100%; display: flex; gap: 8px; align-items: center; padding: 8px; border-radius: var(--radius-sm); color: var(--text-primary); text-align: left; font-size: 11px; }
  .folder-menu-item span:last-child { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .folder-menu-item:hover, .folder-menu-manage:hover { background: var(--bg-glass-hover); }
  .folder-menu-manage { margin-top: 5px; border-top: 1px solid var(--border-subtle); color: var(--text-secondary); }
</style>
