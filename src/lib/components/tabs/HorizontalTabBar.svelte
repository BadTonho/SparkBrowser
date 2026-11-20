<script lang="ts">
  import { onDestroy } from "svelte";
  import { focusShellOnDrag } from "../../actions/focusShellOnDrag";
  import { cancelTabDrag } from "../../stores/tabDragStore";
  import { tabs } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceStore";
  import { splitLayout, splitDragRatio, viewportBounds } from "../../stores/splitViewStore";
  import { splitHeaderGeometry } from "../../services/split-tab-groups.js";
  import WindowControls from "../titlebar/WindowControls.svelte";
  import HorizontalTabList from "./HorizontalTabList.svelte";
  const workspaceTabs = $derived($tabs.filter(tab => !tab.workspace_id || tab.workspace_id === $activeWorkspaceId));
  const geometry = $derived($viewportBounds && $splitLayout ? splitHeaderGeometry($viewportBounds, $splitDragRatio ?? $splitLayout.ratio) : null);
  onDestroy(cancelTabDrag);
</script>
<div class="h-tabbar" class:split={Boolean($splitLayout)} data-tauri-drag-region use:focusShellOnDrag>
  {#if $splitLayout && geometry}
    <div class="pane-tabs" data-testid="split-tabs-left" class:focused={$splitLayout.focused_tab_id === $splitLayout.left_tab_id} style:left="{geometry.left}px" style:width="{geometry.leftWidth}px">
      <HorizontalTabList items={workspaceTabs.filter(tab => $splitLayout!.left_tab_ids.includes(tab.id))} selectedId={$splitLayout.left_tab_id} side="left" />
    </div>
    <div class="pane-tabs right" data-testid="split-tabs-right" class:focused={$splitLayout.focused_tab_id === $splitLayout.right_tab_id} style:left="{geometry.right}px" style:width="{geometry.rightWidth}px">
      <HorizontalTabList items={workspaceTabs.filter(tab => $splitLayout!.right_tab_ids.includes(tab.id))} selectedId={$splitLayout.right_tab_id} side="right" />
    </div>
  {:else}
    <HorizontalTabList items={workspaceTabs} selectedId={workspaceTabs.find(tab => tab.is_active)?.id ?? ''} />
    <div class="drag-spacer" data-tauri-drag-region></div>
  {/if}
  {#if !$splitLayout}<div class="window-controls-host"><WindowControls /></div>{/if}
</div>
<style>
  .h-tabbar { height:38px; flex:0 0 38px; width:100%; position:relative; display:flex; align-items:center; padding:0 114px 0 10px; background:var(--bg-glass-primary); border-bottom:1px solid var(--border-subtle); backdrop-filter:blur(20px); z-index:18; user-select:none; gap:8px; }
  .h-tabbar.split { padding:0; }
  .drag-spacer { flex:1; height:100%; min-width:20px; }
  .pane-tabs { position:absolute; top:0; height:100%; min-width:0; display:flex; border-bottom:2px solid transparent; }
  .pane-tabs.focused { border-bottom-color:var(--accent-primary); }
  .window-controls-host { position:absolute; right:4px; top:6px; }
</style>
