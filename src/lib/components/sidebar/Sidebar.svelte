<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { t } from "../../i18n";
  import WorkspaceSwitcher from "../workspaces/WorkspaceSwitcher.svelte";
  import TabList from "../tabs/TabList.svelte";
  import {
    currentUrl,
    sidebarWidth,
    submitNavigation,
  } from "../../stores/browserStore";
  import {
    bookmarks,
    bookmarksByFolder,
    bookmarkFolders,
    showBookmarkContextMenu,
  } from "../../stores/persistenceStore";
  import { tabLayout } from "../../stores/settingsStore";

  let rootBookmarks = $derived(($bookmarksByFolder.get(null) ?? []));

  function showEmptyContextMenu(event: MouseEvent): void {
    if (event.target instanceof Element && event.target.closest(".quick-card, .bookmark-folder summary")) return;
    showBookmarkContextMenu(event, { kind: "empty" });
  }
</script>

<aside
  class="sidebar"
  style="width: {Math.max(0, $sidebarWidth)}px; opacity: {Math.min(1, $sidebarWidth / 180)};"
>
  <div class="sidebar-inner">
    <!-- Workspaces & Dynamic Themes Switcher -->
    <WorkspaceSwitcher />

    <!-- SQLite Bookmarks Grid -->
    <div class="bookmark-content" role="region" aria-label={$t("sidebar.bookmarks_9631")} oncontextmenu={showEmptyContextMenu}>
      <div class="section-header">
        <span>{$t("sidebar.bookmarks_0b4a")}{$bookmarks.length})</span>
      </div>

    {#if rootBookmarks.length > 0}
      <div class="quick-links-grid">
        {#each rootBookmarks as item (item.id)}
          <button
            class="quick-card"
            class:active={$currentUrl === item.url}
            onclick={() => submitNavigation(item.url)}
            oncontextmenu={(event) => showBookmarkContextMenu(event, { kind: "bookmark", bookmark: item })}
            title={item.url}
          >
            <span class="quick-icon"><AppIcon name={item.icon} /></span>
            <span class="quick-title">{item.title}</span>
          </button>
        {/each}
      </div>
    {/if}

    {#each $bookmarkFolders as folder (folder.id)}
      {@const folderBookmarks = ($bookmarksByFolder.get(folder.id) ?? [])}
      <details class="bookmark-folder" open={false}>
        <summary oncontextmenu={(event) => showBookmarkContextMenu(event, { kind: "folder", folder })}>
          <span aria-hidden="true"><AppIcon name="folder" /></span><span>{folder.name}</span><span class="folder-count">{folderBookmarks.length}</span>
        </summary>
        {#if folderBookmarks.length > 0}
          <div class="quick-links-grid folder-links">
            {#each folderBookmarks as item (item.id)}
              <button
                class="quick-card"
                class:active={$currentUrl === item.url}
                onclick={() => submitNavigation(item.url)}
                oncontextmenu={(event) => showBookmarkContextMenu(event, { kind: "bookmark", bookmark: item })}
                title={item.url}
              >
                <span class="quick-icon"><AppIcon name={item.icon} /></span>
                <span class="quick-title">{item.title}</span>
              </button>
            {/each}
          </div>
        {:else}
          <div class="folder-empty">{$t("sidebar.no_bookmarks_in_this_folder_3a58")}</div>
        {/if}
      </details>
    {/each}

      {#if $bookmarks.length === 0 && $bookmarkFolders.length === 0}
        <div class="folder-empty">{$t("sidebar.no_bookmarks_yet_dac0")}</div>
      {/if}
    </div>

    <!-- Multi-Tab System with Workspace Filtering & Hibernation (Sidebar mode) -->
    {#if $tabLayout === "vertical"}
      <TabList />
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    height: 100%;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--bg-glass-primary);
    border-right: 1px solid var(--border-subtle);
    backdrop-filter: blur(24px);
    will-change: width, opacity;
  }

  .sidebar-inner {
    width: 276px;
    height: 100%;
    padding: 12px 11px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.8px;
    color: var(--text-muted);
    padding: 0 4px;
  }

  .bookmark-content { min-height: 0; max-height: 40%; overflow-y: auto; scrollbar-width: thin; }

  .bookmark-folder { min-width: 0; }
  .bookmark-folder summary { display: flex; align-items: center; gap: 7px; padding: 5px 4px; color: var(--text-secondary); font-size: 11px; cursor: pointer; list-style: none; }
  .bookmark-folder summary::-webkit-details-marker { display: none; }
  .bookmark-folder summary:hover { color: var(--text-primary); }
  .folder-count { margin-left: auto; color: var(--text-muted); font-size: 10px; }
  .folder-links { margin-top: 4px; }
  .folder-empty { padding: 5px 8px; color: var(--text-muted); font-size: 10px; }

  .quick-links-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
    max-height: 82px;
    overflow-y: auto;
  }

  .quick-card {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 9px;
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--border-subtle);
    transition:
      background 0.2s ease,
      transform 0.18s cubic-bezier(0.22, 1, 0.36, 1),
      border-color 0.2s ease;
  }

  .quick-card:hover {
    background: var(--bg-glass-hover);
    border-color: var(--accent-primary);
    transform: translateY(-1px);
  }

  .quick-card.active {
    background: color-mix(in srgb, var(--accent-primary) 16%, transparent);
    border-color: var(--accent-primary);
  }

  .quick-icon {
    font-size: 13.5px;
  }

  .quick-title {
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
