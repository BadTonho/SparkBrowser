<script lang="ts">
  import { bookmarks } from "../../stores/persistenceStore";
  import { currentUrl, submitNavigation } from "../../stores/browserStore";
</script>

<nav class="bookmark-bar" aria-label="Bookmarks">
  {#if $bookmarks.length === 0}
    <span class="bookmark-empty">No bookmarks yet</span>
  {:else}
    <div class="bookmark-list">
      {#each $bookmarks as item (item.id)}
        <button
          class="bookmark-item"
          class:active={$currentUrl === item.url}
          onclick={() => submitNavigation(item.url)}
          title={item.url}
          aria-label="Open bookmark: {item.title}"
        >
          <span class="bookmark-icon" aria-hidden="true">{item.icon}</span>
          <span class="bookmark-title">{item.title}</span>
        </button>
      {/each}
    </div>
  {/if}
</nav>

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

  .bookmark-list::-webkit-scrollbar {
    display: none;
  }

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

  .bookmark-item:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
  }

  .bookmark-item.active {
    color: var(--accent-primary);
    background: color-mix(in srgb, var(--accent-primary) 14%, transparent);
  }

  .bookmark-icon {
    flex: 0 0 auto;
  }

  .bookmark-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .bookmark-empty {
    padding: 4px 8px;
    color: var(--text-muted);
    font-size: 11px;
  }
</style>
