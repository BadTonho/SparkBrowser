<script lang="ts">
  import { t } from "../../i18n";
  import type { ExtensionDetails } from "../../types/extensions";
  import { toggleExtension, removeExtension, openExtensionPopup } from "../../stores/extensionsStore";

  interface Props {
    extension: ExtensionDetails;
  }

  let { extension }: Props = $props();
  let showDetails = $state(false);

  async function handleToggle() {
    await toggleExtension(extension.id, !extension.enabled);
  }

  async function handleRemove() {
    if (confirm($t("extensions.remove_confirm"))) {
      await removeExtension(extension.id);
    }
  }
</script>

<div class="extension-card" class:disabled={!extension.enabled}>
  <div class="extension-main">
    <div class="extension-icon-box">
      {#if extension.icon_data_url}
        <img src={extension.icon_data_url} alt="" class="extension-icon" />
      {:else}
        <svg class="extension-fallback-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z" />
        </svg>
      {/if}
    </div>

    <div class="extension-info">
      <div class="extension-title-row">
        <h3 class="extension-name">{extension.name}</h3>
        <span class="extension-version">v{extension.version}</span>
      </div>
      <p class="extension-desc">{extension.description || "—"}</p>
      
      <div class="extension-id-row">
        <span class="extension-id">{$t("extensions.id")}: {extension.id}</span>
      </div>
    </div>

    <div class="extension-controls">
      <label class="toggle-switch" title={extension.enabled ? $t("extensions.enabled") : $t("extensions.disabled")}>
        <input type="checkbox" checked={extension.enabled} onchange={handleToggle} />
        <span class="slider"></span>
      </label>

      {#if extension.enabled && extension.popup_url}
        <button
          type="button"
          class="icon-action-btn"
          title={$t("extensions.open_popup")}
          onclick={() => openExtensionPopup(extension)}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
            <polyline points="15 3 21 3 21 9" />
            <line x1="10" y1="14" x2="21" y2="3" />
          </svg>
        </button>
      {/if}

      <button
        type="button"
        class="icon-action-btn details-btn"
        class:active={showDetails}
        title="Detalhes"
        onclick={() => (showDetails = !showDetails)}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="16" x2="12" y2="12" />
          <line x1="12" y1="8" x2="12.01" y2="8" />
        </svg>
      </button>

      <button
        type="button"
        class="icon-action-btn remove-btn"
        title={$t("extensions.remove")}
        onclick={handleRemove}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16">
          <polyline points="3 6 5 6 21 6" />
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
        </svg>
      </button>
    </div>
  </div>

  {#if showDetails}
    <div class="extension-details-panel">
      <div class="details-section">
        <strong>{$t("extensions.permissions")}:</strong>
        {#if extension.permissions && extension.permissions.length > 0}
          <div class="permissions-tags">
            {#each extension.permissions as perm}
              <span class="perm-tag">{perm}</span>
            {/each}
          </div>
        {:else}
          <p class="perm-empty">{$t("extensions.no_permissions")}</p>
        {/if}
      </div>

      <div class="details-section path-section">
        <strong>Path:</strong>
        <code class="path-code">{extension.path}</code>
      </div>
    </div>
  {/if}
</div>

<style>
  .extension-card {
    background: var(--bg-surface-elevated, rgba(255, 255, 255, 0.05));
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
    border-radius: 8px;
    padding: 14px 16px;
    transition: border-color 0.15s ease, opacity 0.15s ease;
  }

  .extension-card.disabled {
    opacity: 0.65;
  }

  .extension-main {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .extension-icon-box {
    width: 44px;
    height: 44px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.04);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    overflow: hidden;
  }

  .extension-icon {
    width: 36px;
    height: 36px;
    object-fit: contain;
  }

  .extension-fallback-icon {
    width: 24px;
    height: 24px;
    opacity: 0.6;
  }

  .extension-info {
    flex: 1;
    min-width: 0;
  }

  .extension-title-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .extension-name {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .extension-version {
    font-size: 11px;
    color: var(--text-muted, rgba(255, 255, 255, 0.5));
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 6px;
    border-radius: 4px;
  }

  .extension-desc {
    margin: 4px 0 0;
    font-size: 12.5px;
    color: var(--text-secondary, rgba(255, 255, 255, 0.7));
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .extension-id-row {
    margin-top: 4px;
  }

  .extension-id {
    font-size: 11px;
    font-family: monospace;
    color: var(--text-muted, rgba(255, 255, 255, 0.4));
  }

  .extension-controls {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  /* Toggle Switch */
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 40px;
    height: 22px;
    cursor: pointer;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    inset: 0;
    background-color: rgba(255, 255, 255, 0.2);
    border-radius: 22px;
    transition: background-color 0.2s;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 16px;
    width: 16px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    border-radius: 50%;
    transition: transform 0.2s;
  }

  input:checked + .slider {
    background-color: var(--accent-primary, #3b82f6);
  }

  input:checked + .slider:before {
    transform: translateX(18px);
  }

  .icon-action-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary, rgba(255, 255, 255, 0.7));
    cursor: pointer;
    padding: 6px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
  }

  .icon-action-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
  }

  .icon-action-btn.remove-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .details-btn.active {
    background: rgba(255, 255, 255, 0.15);
    color: var(--accent-primary, #3b82f6);
  }

  .extension-details-panel {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
    font-size: 12px;
  }

  .details-section {
    margin-bottom: 8px;
  }

  .permissions-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }

  .perm-tag {
    background: rgba(59, 130, 246, 0.15);
    color: #93c5fd;
    padding: 2px 8px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 11px;
  }

  .perm-empty {
    margin: 4px 0 0;
    color: var(--text-muted, rgba(255, 255, 255, 0.5));
  }

  .path-code {
    display: block;
    margin-top: 4px;
    padding: 4px 8px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 4px;
    font-size: 11px;
    word-break: break-all;
    color: var(--text-muted, rgba(255, 255, 255, 0.6));
  }
</style>
