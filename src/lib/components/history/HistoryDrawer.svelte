<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import {
    historyEntries,
    isHistoryDrawerOpen,
    closeHistoryDrawer,
    wipeBrowsingHistory,
  } from "../../stores/persistenceStore";
  import { addTab } from "../../stores/browserStore";

  let filterQuery = $state("");

  let filteredHistory = $derived(
    $historyEntries.filter((entry) => {
      const q = filterQuery.trim().toLowerCase();
      if (!q) return true;
      return (
        entry.title.toLowerCase().includes(q) ||
        entry.url.toLowerCase().includes(q)
      );
    })
  );

  function formatTime(epochSecs: number): string {
    if (!epochSecs) return "";
    const date = new Date(epochSecs * 1000);
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

  async function openHistoryUrl(url: string) {
    await closeHistoryDrawer();
    await addTab(url);
  }
</script>

{#if $isHistoryDrawerOpen}
  <div
    class="history-backdrop"
    transition:fade={{ duration: 150 }}
    onclick={() => closeHistoryDrawer()}
    onkeydown={(e) => e.key === "Escape" && closeHistoryDrawer()}
    role="presentation"
  >
    <div
      class="history-panel"
      in:fly={{ x: -28, duration: 220, easing: cubicOut }}
      out:fade={{ duration: 130 }}
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-label="Browsing History"
      tabindex="-1"
    >
      <div class="history-header">
        <div class="header-title">
          <span>🕒 Browsing History (SQLite)</span>
        </div>
        <div class="header-actions">
          {#if $historyEntries.length > 0}
            <button class="clear-btn" onclick={wipeBrowsingHistory}>
              Clear All
            </button>
          {/if}
          <button class="close-btn" onclick={() => closeHistoryDrawer()}>
            ✕
          </button>
        </div>
      </div>

      <div class="history-search">
        <input
          type="text"
          bind:value={filterQuery}
          placeholder="Search visited pages..."
          spellcheck="false"
        />
      </div>

      <div class="history-list">
        {#if filteredHistory.length === 0}
          <div class="empty-state">
            <p>No history entries found.</p>
          </div>
        {:else}
          {#each filteredHistory as item (item.id)}
            <button
              class="history-row"
              onclick={() => openHistoryUrl(item.url)}
            >
              <div class="history-meta">
                <span class="history-title">{item.title}</span>
                <span class="history-url">{item.url}</span>
              </div>
              <span class="history-time">{formatTime(item.visited_at)}</span>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .history-backdrop {
    position: fixed;
    inset: 0;
    z-index: 990;
    background: rgba(8, 8, 8, 0.55);
    backdrop-filter: blur(12px);
    display: flex;
    justify-content: flex-start;
    padding: 46px 0 12px 12px;
  }

  .history-panel {
    width: min(420px, 90vw);
    height: 100%;
    border-radius: var(--radius-lg);
    background: rgba(18, 18, 18, 0.95);
    border: 1px solid color-mix(in srgb, var(--accent-primary) 35%, transparent);
    box-shadow: 0 20px 56px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .history-header {
    padding: 14px 16px;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-title {
    font-size: 13.5px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .clear-btn {
    padding: 4px 9px;
    border-radius: 6px;
    background: rgba(232, 17, 35, 0.16);
    color: #ff7b86;
    font-size: 11px;
    font-weight: 600;
  }

  .clear-btn:hover {
    background: rgba(232, 17, 35, 0.28);
  }

  .close-btn {
    width: 26px;
    height: 26px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .close-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
  }

  .history-search {
    padding: 10px 14px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .history-search input {
    width: 100%;
    padding: 8px 11px;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    font-size: 12.5px;
    color: var(--text-primary);
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .empty-state {
    padding: 36px 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12.5px;
  }

  .history-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 9px 11px;
    border-radius: var(--radius-md);
    text-align: left;
    transition: background 0.15s ease, transform 0.15s ease;
  }

  .history-row:hover {
    background: var(--bg-glass-hover);
    transform: translateX(2px);
  }

  .history-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .history-title {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .history-url {
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .history-time {
    font-size: 10.5px;
    color: var(--text-muted);
    flex-shrink: 0;
  }
</style>
