<script lang="ts">
  import { onDestroy } from "svelte";
  import { cancelTabDrag } from "../../stores/tabDragStore";
  import { tabs } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceStore";
  import { splitLayout, splitDragRatio, viewportBounds } from "../../stores/splitViewStore";
  import { splitHeaderGeometry } from "../../services/split-tab-groups.js";
  import { minimizeWindow, toggleMaximizeWindow, closeWindow } from "../../services/tauri-bridge";
  import { t } from "../../i18n";
  import HorizontalTabList from "./HorizontalTabList.svelte";
  const workspaceTabs = $derived($tabs.filter(tab => !tab.workspace_id || tab.workspace_id === $activeWorkspaceId));
  const geometry = $derived($viewportBounds && $splitLayout ? splitHeaderGeometry($viewportBounds, $splitDragRatio ?? $splitLayout.ratio) : null);
  onDestroy(cancelTabDrag);
</script>
<div class="h-tabbar" data-tauri-drag-region>
  {#if $splitLayout && geometry}
    <div class="pane-tabs" data-testid="split-tabs-left" class:focused={$splitLayout.focused_tab_id === $splitLayout.left_tab_id} style:left="{geometry.left}px" style:width="{geometry.leftWidth}px">
      <HorizontalTabList items={workspaceTabs.filter(tab => $splitLayout!.left_tab_ids.includes(tab.id))} selectedId={$splitLayout.left_tab_id} side="left" />
    </div>
    <div class="pane-tabs right" data-testid="split-tabs-right" class:focused={$splitLayout.focused_tab_id === $splitLayout.right_tab_id} style:left="{geometry.right}px" style:width="min({geometry.rightWidth}px, calc(100% - {geometry.right + 114}px))">
      <HorizontalTabList items={workspaceTabs.filter(tab => $splitLayout!.right_tab_ids.includes(tab.id))} selectedId={$splitLayout.right_tab_id} side="right" />
    </div>
  {:else}
    <HorizontalTabList items={workspaceTabs} selectedId={workspaceTabs.find(tab => tab.is_active)?.id ?? ''} />
    <div class="drag-spacer" data-tauri-drag-region></div>
  {/if}
  <div class="window-controls">
    <button class="win-btn" onclick={minimizeWindow} title={$t("tabs.minimize_d72e")} aria-label={$t("tabs.minimize_d72e")}>
      <svg width="12" height="12" viewBox="0 0 12 12"><rect x="2" y="5.5" width="8" height="1" fill="currentColor" /></svg>
    </button>
    <button class="win-btn" onclick={toggleMaximizeWindow} title={$t("tabs.maximize_fba6")} aria-label={$t("tabs.maximize_fba6")}>
      <svg width="12" height="12" viewBox="0 0 12 12"><rect x="2.5" y="2.5" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.2" fill="none" /></svg>
    </button>
    <button class="win-btn close-btn" onclick={closeWindow} title={$t("tabs.close_7d9e")} aria-label={$t("tabs.close_7d9e")}>
      <svg width="12" height="12" viewBox="0 0 12 12"><path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" /></svg>
    </button>
  </div>
</div>
<style>
  .h-tabbar { height:38px; flex:0 0 38px; width:100%; position:relative; display:flex; align-items:center; padding:0 114px 0 10px; background:var(--bg-glass-primary); border-bottom:1px solid var(--border-subtle); backdrop-filter:blur(20px); z-index:18; user-select:none; gap:8px; }
  .drag-spacer { flex:1; height:100%; min-width:20px; }
  .pane-tabs { position:absolute; top:0; height:100%; min-width:0; display:flex; border-bottom:2px solid transparent; }
  .pane-tabs.focused { border-bottom-color:var(--accent-primary); }
  .window-controls {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .win-btn {
    width: 34px;
    height: 26px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    transition: background 0.15s ease, color 0.15s ease;
  }

  .win-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
  }

  .win-btn.close-btn:hover {
    background: #e81123;
    color: #ffffff;
  }


  .window-controls { position:absolute; right:4px; top:6px; }
</style>
