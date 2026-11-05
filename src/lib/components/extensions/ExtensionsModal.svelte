<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../../i18n";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { onOverlayOpening } from "../../services/bridge/overlays";
  import {
    installedExtensions,
    isExtensionsModalOpen,
    closeExtensionsModal,
    installUnpacked,
    installFromWebStore,
    installArchive,
    extensionsLoading,
    extensionsError,
  } from "../../stores/extensionsStore";
  import ExtensionItem from "./ExtensionItem.svelte";

  let searchQuery = $state("");
  let webstoreInput = $state("");
  let isInstallingWebStore = $state(false);

  onMount(() => {
    return onOverlayOpening((id) => {
      if (id !== "extensions") void closeExtensionsModal();
    });
  });

  const filtered = $derived(
    $installedExtensions.filter((ext) => {
      if (!searchQuery.trim()) return true;
      const q = searchQuery.toLowerCase();
      return (
        ext.name.toLowerCase().includes(q) ||
        ext.description.toLowerCase().includes(q) ||
        ext.id.toLowerCase().includes(q)
      );
    })
  );

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeExtensionsModal();
    }
  }

  async function handleLoadUnpacked() {
    await installUnpacked();
  }

  async function handleLoadArchive() {
    await installArchive();
  }

  async function handleInstallWebStore() {
    if (!webstoreInput.trim() || isInstallingWebStore) return;
    try {
      isInstallingWebStore = true;
      const res = await installFromWebStore(webstoreInput.trim());
      if (res) {
        webstoreInput = "";
      }
    } finally {
      isInstallingWebStore = false;
    }
  }

  function handleWebStoreKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      void handleInstallWebStore();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $isExtensionsModalOpen}
  <div
    class="modal-backdrop"
    role="presentation"
    transition:fade={{ duration: 150 }}
    onclick={closeExtensionsModal}
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="modal-window"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="extensions-modal-title"
      transition:fly={{ y: 20, duration: 200, easing: cubicOut }}
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="modal-header">
        <div class="header-left">
          <svg class="header-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z" />
          </svg>
          <h2 id="extensions-modal-title">{$t("extensions.title")}</h2>
          <span class="extension-count">{$installedExtensions.length}</span>
        </div>

        <div class="header-actions">
          <button
            type="button"
            class="action-btn secondary-btn"
            disabled={$extensionsLoading || isInstallingWebStore}
            onclick={handleLoadArchive}
            title="Instalar extensão a partir de arquivo .zip ou .crx"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="15" height="15">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
              <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
              <line x1="12" y1="22.08" x2="12" y2="12" />
            </svg>
            <span>{$t("extensions.load_archive")}</span>
          </button>

          <button
            type="button"
            class="action-btn primary-btn"
            disabled={$extensionsLoading || isInstallingWebStore}
            onclick={handleLoadUnpacked}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="15" height="15">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            <span>{$t("extensions.load_unpacked")}</span>
          </button>

          <button
            type="button"
            class="close-btn"
            title="Fechar"
            onclick={closeExtensionsModal}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>

      <!-- WebStore Install Row -->
      <div class="webstore-install-row">
        <div class="webstore-input-wrapper">
          <svg class="webstore-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
            <circle cx="12" cy="12" r="10" />
            <circle cx="12" cy="12" r="4" />
            <line x1="21.17" y1="8" x2="12" y2="8" />
            <line x1="3.95" y1="6.06" x2="8.54" y2="14" />
            <line x1="10.88" y1="21.94" x2="15.46" y2="14" />
          </svg>
          <input
            type="text"
            class="webstore-input"
            placeholder={$t("extensions.webstore_placeholder")}
            bind:value={webstoreInput}
            onkeydown={handleWebStoreKeydown}
            disabled={$extensionsLoading || isInstallingWebStore}
          />
        </div>
        <button
          type="button"
          class="webstore-btn"
          disabled={!webstoreInput.trim() || $extensionsLoading || isInstallingWebStore}
          onclick={handleInstallWebStore}
        >
          {#if isInstallingWebStore}
            <div class="spinner"></div>
            <span>{$t("extensions.installing")}</span>
          {:else}
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            <span>{$t("extensions.install_webstore")}</span>
          {/if}
        </button>
      </div>

      <!-- Search & Error banner -->
      <div class="modal-subbar">
        <div class="search-box">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            type="search"
            placeholder={$t("extensions.search_placeholder")}
            bind:value={searchQuery}
          />
        </div>
      </div>

      {#if $extensionsError}
        <div class="error-banner">
          {$extensionsError}
        </div>
      {/if}

      <!-- Extensions list -->
      <div class="modal-body">
        {#if filtered.length > 0}
          <div class="extensions-grid">
            {#each filtered as ext (ext.id)}
              <ExtensionItem extension={ext} />
            {/each}
          </div>
        {:else}
          <div class="empty-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" width="48" height="48">
              <path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z" />
            </svg>
            <p class="empty-title">{$t("extensions.no_extensions")}</p>
            <p class="empty-hint">{$t("extensions.no_extensions_hint")}</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 24px;
  }

  .modal-window {
    background: var(--bg-surface, #1e1e24);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.12));
    border-radius: 12px;
    width: 100%;
    max-width: 680px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 24px;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .header-icon {
    width: 24px;
    height: 24px;
    color: var(--accent-primary, #3b82f6);
  }

  .header-left h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary, #ffffff);
  }

  .extension-count {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-secondary, rgba(255, 255, 255, 0.7));
    font-size: 12px;
    padding: 2px 8px;
    border-radius: 12px;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 12px;
    border-radius: 6px;
    font-size: 12.5px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s, background 0.15s;
    border: none;
    white-space: nowrap;
  }

  .primary-btn {
    background: var(--accent-primary, #3b82f6);
    color: #ffffff;
  }

  .primary-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .secondary-btn {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: var(--text-primary, #ffffff);
  }

  .secondary-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.14);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted, rgba(255, 255, 255, 0.5));
    cursor: pointer;
    padding: 6px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
    margin-left: 4px;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
  }

  .webstore-install-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 24px;
    background: rgba(255, 255, 255, 0.02);
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.06));
  }

  .webstore-input-wrapper {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 6px 12px;
  }

  .webstore-icon {
    color: var(--text-muted, rgba(255, 255, 255, 0.4));
    flex-shrink: 0;
  }

  .webstore-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary, #ffffff);
    font-size: 13px;
  }

  .webstore-input::placeholder {
    color: var(--text-muted, rgba(255, 255, 255, 0.35));
  }

  .webstore-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #10b981;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 7px 14px;
    font-size: 12.5px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, opacity 0.15s;
    flex-shrink: 0;
  }

  .webstore-btn:hover:not(:disabled) {
    background: #059669;
  }

  .webstore-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #ffffff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .modal-subbar {
    padding: 12px 24px;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.05));
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 6px 12px;
    color: var(--text-muted, rgba(255, 255, 255, 0.5));
  }

  .search-box input {
    background: transparent;
    border: none;
    outline: none;
    width: 100%;
    color: var(--text-primary, #ffffff);
    font-size: 13px;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border-left: 3px solid #ef4444;
    color: #fca5a5;
    padding: 10px 24px;
    font-size: 12.5px;
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px 24px 24px;
  }

  .extensions-grid {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 16px;
    color: var(--text-muted, rgba(255, 255, 255, 0.4));
    text-align: center;
  }

  .empty-title {
    margin: 16px 0 4px;
    font-size: 15px;
    font-weight: 500;
    color: var(--text-secondary, rgba(255, 255, 255, 0.7));
  }

  .empty-hint {
    margin: 0;
    font-size: 13px;
  }
</style>
