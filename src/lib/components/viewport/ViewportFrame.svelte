<script lang="ts">
  import { t } from "../../i18n";
  import { isSitePermissionsOpen, permissionRequests } from "../../stores/sitePermissionsStore";
  import SplitDivider from "./SplitDivider.svelte";
  import { splitLayout, splitDragRatio, splitKey, viewportBounds } from "../../stores/splitViewStore";
  import { isToolsOpen } from "../../stores/toolsStore";
  import { onMount } from "svelte";
  import {
    tabs,
    sidebarWidth,
    currentUrl,
  } from "../../stores/browserStore";
  import { isCommandBarOpen } from "../../stores/commandBarStore";
  import { isHistoryDrawerOpen } from "../../stores/persistenceStore";
  import { isDownloadsDrawerOpen } from "../../stores/downloadsStore";
  import { isSettingsModalOpen, isZenMode } from "../../stores/settingsStore";
  import { isWorkspaceSwitcherOverlayOpen } from "../../stores/workspaceSwitcherStore";
  import { titlebarMenu } from "../../stores/titlebarMenuStore";
  import { syncViewportBounds, isTauriRuntime } from "../../services/tauri-bridge";
  import { createViewportSync } from "../../services/viewport-sync.js";

  let viewportEl: HTMLDivElement | undefined = $state();
  const inTauri = isTauriRuntime();

  let anyOverlayOpen = $derived(
    $isSitePermissionsOpen || $permissionRequests.length > 0 || $isToolsOpen || $isCommandBarOpen ||
      $isHistoryDrawerOpen ||
      $isDownloadsDrawerOpen ||
      $isSettingsModalOpen ||
      $isWorkspaceSwitcherOverlayOpen || Boolean($titlebarMenu)
  );

  const viewportSync = createViewportSync(() => {
    if (!viewportEl) return;
    const rect = viewportEl.getBoundingClientRect();
    viewportBounds.set({ x: Math.round(rect.left), y: Math.round(rect.top), width: Math.round(rect.width), height: Math.round(rect.height) });
    return {
      split_ratio: $splitLayout ? $splitDragRatio ?? $splitLayout.ratio : undefined,
      split_key: $splitLayout ? splitKey($splitLayout) : undefined,
      x: Math.round(rect.left),
      y: Math.round(rect.top),
      width: Math.max(100, Math.round(rect.width)),
      height: Math.max(100, Math.round(rect.height)),
    };
  }, syncViewportBounds, (error) => console.error("Viewport resize failed", error));
  const measureAndSync = () => viewportSync.schedule();

  $effect(() => {
    void $splitLayout;
    void $splitDragRatio;
    void $sidebarWidth;
    void $isZenMode;
    measureAndSync();
  });

  onMount(() => {
    measureAndSync();

    const resizeObserver = new ResizeObserver(() => {
      measureAndSync();
    });

    if (viewportEl) {
      resizeObserver.observe(viewportEl);
    }

    window.addEventListener("resize", measureAndSync);
    return () => {
      resizeObserver.disconnect();
      viewportSync.dispose();
      window.removeEventListener("resize", measureAndSync);
    };
  });
</script>

<main class="viewport-shell" class:zen={$isZenMode}>
  <div class="viewport-frame" bind:this={viewportEl}>
    {#if inTauri && !anyOverlayOpen && viewportEl && $splitLayout}<SplitDivider frame={viewportEl} flush={viewportSync.flush} />{/if}
    {#if !inTauri || anyOverlayOpen}
      {#if (!inTauri || !$isWorkspaceSwitcherOverlayOpen) && !$titlebarMenu}
        <div class="browser-preview-fallback">
          <div class="fallback-card">
            <h2>{$t("viewport.multi_webview_viewport_5f2a")}{$tabs.length} {$tabs.length === 1 ? $t("viewport.tab_90dd") : $t("viewport.tabs_8e5e")})</h2>
            <p>{$t("viewport.active_url_fa41")} <strong>{$currentUrl}</strong></p>
            <p class="hint">
              {#if anyOverlayOpen}
                {$t("viewport.overlay_active_press_bf4f")} <kbd>ESC</kbd> {$t("viewport.to_return_to_the_active_webview_9829")}
              {:else}
                {$t("viewport.run_via_3e45")} <code>npm run tauri dev</code> {$t("viewport.to_mount_the_native_rust_child_webview_inside_this_frame_5fdc")}
              {/if}
            </p>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</main>

<style>
  .viewport-shell {
    flex: 1;
    height: 100%;
    padding: 8px;
    background: var(--bg-glass-primary);
    display: flex;
    transition: padding 0.2s ease;
  }

  .viewport-shell.zen {
    padding: 0;
  }

  .viewport-frame {
    flex: 1;
    width: 100%;
    height: 100%;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    background: #101010;
    box-shadow: var(--shadow-soft);
    overflow: hidden;
    position: relative;
  }

  .viewport-shell.zen .viewport-frame {
    border-radius: 0;
    border: none;
  }

  .browser-preview-fallback {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    text-align: center;
  }

  .fallback-card {
    max-width: 440px;
    padding: 28px;
    border-radius: var(--radius-lg);
    background: var(--bg-glass-elevated);
    border: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .fallback-card h2 {
    font-size: 17px;
  }

  .fallback-card p {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .fallback-card .hint {
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
