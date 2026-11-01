<script lang="ts">
  import WorkspaceSwitcher from "../workspaces/WorkspaceSwitcher.svelte";
  import TabList from "../tabs/TabList.svelte";
  import {
    currentUrl,
    sidebarWidth,
    submitNavigation,
  } from "../../stores/browserStore";
  import { bookmarks } from "../../stores/persistenceStore";
  import { tabLayout } from "../../stores/settingsStore";
</script>

<aside
  class="sidebar"
  style="width: {Math.max(0, $sidebarWidth)}px; opacity: {Math.min(1, $sidebarWidth / 180)};"
>
  <div class="sidebar-inner">
    <!-- Workspaces & Dynamic Themes Switcher -->
    <WorkspaceSwitcher />

    <!-- SQLite Bookmarks Grid -->
    <div class="section-header">
      <span>BOOKMARKS ({$bookmarks.length})</span>
    </div>

    <div class="quick-links-grid">
      {#each $bookmarks as item (item.id)}
        <button
          class="quick-card"
          class:active={$currentUrl === item.url}
          onclick={() => submitNavigation(item.url)}
          title={item.url}
        >
          <span class="quick-icon">{item.icon}</span>
          <span class="quick-title">{item.title}</span>
        </button>
      {/each}
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
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.8px;
    color: var(--text-muted);
    padding: 0 4px;
  }

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
