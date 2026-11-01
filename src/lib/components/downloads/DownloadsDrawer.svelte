<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import {
    downloads,
    isDownloadsDrawerOpen,
    closeDownloadsDrawer,
    startDownloadFromUrl,
    wipeDownloads,
  } from "../../stores/downloadsStore";
  import { currentUrl } from "../../stores/browserStore";

  let downloadUrlInput = $state("");

  async function handleQuickDownload() {
    const target = downloadUrlInput.trim() || $currentUrl;
    if (!target) return;
    await startDownloadFromUrl(target);
    downloadUrlInput = "";
  }
</script>

{#if $isDownloadsDrawerOpen}
  <div
    class="downloads-backdrop"
    transition:fade={{ duration: 150 }}
    onclick={() => closeDownloadsDrawer()}
    onkeydown={(e) => e.key === "Escape" && closeDownloadsDrawer()}
    role="presentation"
  >
    <div
      class="downloads-panel"
      in:fly={{ x: -28, duration: 220, easing: cubicOut }}
      out:fade={{ duration: 130 }}
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-label="Download Manager"
      tabindex="-1"
    >
      <div class="downloads-header">
        <div class="header-title">
          <span>📥 Download Manager</span>
        </div>
        <div class="header-actions">
          {#if $downloads.length > 0}
            <button class="clear-btn" onclick={wipeDownloads}>
              Clear List
            </button>
          {/if}
          <button class="close-btn" onclick={() => closeDownloadsDrawer()}>
            ✕
          </button>
        </div>
      </div>

      <div class="quick-download-bar">
        <input
          type="text"
          bind:value={downloadUrlInput}
          placeholder="Paste direct file URL or leave empty to save active page..."
          onkeydown={(e) => e.key === "Enter" && handleQuickDownload()}
          spellcheck="false"
        />
        <button class="download-btn" onclick={handleQuickDownload}>
          Download
        </button>
      </div>

      <div class="downloads-list">
        {#if $downloads.length === 0}
          <div class="empty-state">
            <p>No downloads recorded yet.</p>
            <span class="empty-sub">Press Download above to save a resource to your OS Downloads folder.</span>
          </div>
        {:else}
          {#each $downloads as item (item.id)}
            <div class="download-card">
              <span class="file-icon">📄</span>
              <div class="download-meta">
                <span class="file-name">{item.file_name}</span>
                <span class="file-path">{item.save_path}</span>
              </div>
              <span class="status-badge">{item.status}</span>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .downloads-backdrop {
    position: fixed;
    inset: 0;
    z-index: 990;
    background: rgba(8, 8, 8, 0.55);
    backdrop-filter: blur(12px);
    display: flex;
    justify-content: flex-start;
    padding: 46px 0 12px 12px;
  }

  .downloads-panel {
    width: min(450px, 92vw);
    height: 100%;
    border-radius: var(--radius-lg);
    background: rgba(18, 18, 18, 0.95);
    border: 1px solid color-mix(in srgb, var(--accent-primary) 35%, transparent);
    box-shadow: 0 20px 56px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .downloads-header {
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

  .quick-download-bar {
    padding: 10px 14px;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    gap: 8px;
  }

  .quick-download-bar input {
    flex: 1;
    padding: 8px 11px;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    font-size: 12px;
    color: var(--text-primary);
  }

  .download-btn {
    padding: 0 12px;
    border-radius: var(--radius-sm);
    background: var(--accent-gradient);
    color: var(--accent-foreground);
    font-size: 11.5px;
    font-weight: 600;
  }

  .downloads-list {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .empty-state {
    padding: 36px 16px;
    text-align: center;
    color: var(--text-muted);
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12.5px;
  }

  .empty-sub {
    font-size: 11px;
  }

  .download-card {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--border-subtle);
  }

  .file-icon {
    font-size: 18px;
  }

  .download-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .file-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-path {
    font-size: 10.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-badge {
    padding: 3px 8px;
    border-radius: 99px;
    background: rgba(34, 197, 94, 0.16);
    color: #4ade80;
    font-size: 10px;
    font-weight: 600;
  }
</style>

