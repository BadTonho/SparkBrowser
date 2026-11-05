<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { activeExtensionPopup, closeExtensionPopup } from "../../stores/extensionsStore";

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeExtensionPopup();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $activeExtensionPopup}
  <div
    class="popover-backdrop"
    role="presentation"
    transition:fade={{ duration: 120 }}
    onclick={closeExtensionPopup}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="extension-popover-card"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-label={$activeExtensionPopup.name}
      transition:fly={{ y: -10, duration: 150, easing: cubicOut }}
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <div class="popover-header">
        <span class="extension-name">{$activeExtensionPopup.name}</span>
        <button type="button" class="close-btn" onclick={closeExtensionPopup} title="Fechar">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <div class="popover-frame-container">
        <iframe
          src={$activeExtensionPopup.popup_url}
          title={$activeExtensionPopup.name}
          class="popover-iframe"
          sandbox="allow-scripts allow-forms allow-same-origin allow-popups"
        ></iframe>
      </div>
    </div>
  </div>
{/if}

<style>
  .popover-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9998;
    background: transparent;
  }

  .extension-popover-card {
    position: fixed;
    top: 48px;
    right: 60px;
    width: 360px;
    max-width: 90vw;
    height: 480px;
    max-height: 75vh;
    background: var(--bg-surface, #1e1e24);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.15));
    border-radius: 10px;
    box-shadow: 0 16px 32px rgba(0, 0, 0, 0.45);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    z-index: 9999;
  }

  .popover-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.04);
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
    user-select: none;
  }

  .extension-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary, rgba(255, 255, 255, 0.8));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted, rgba(255, 255, 255, 0.5));
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
  }

  .popover-frame-container {
    flex: 1;
    width: 100%;
    background: #ffffff;
    position: relative;
  }

  .popover-iframe {
    width: 100%;
    height: 100%;
    border: none;
  }
</style>
