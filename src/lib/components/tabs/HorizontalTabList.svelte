<script lang="ts">
  import type { TabItem } from "../../types/browser";
  import { addTab } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceStore";
  import { t } from "../../i18n";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import HorizontalTabItem from "./HorizontalTabItem.svelte";
  let { items, selectedId, side = null }: { items: TabItem[]; selectedId: string; side?: "left" | "right" | null } = $props();
</script>

<div class="tabs-scroll" data-pane-side={side ?? undefined} onwheel={(event) => { event.currentTarget.scrollLeft += event.deltaY; }} data-tauri-drag-region aria-label={side ? $t(side === "left" ? "tabs.left_pane_tabs" : "tabs.right_pane_tabs") : undefined}>
  {#each items as tab (tab.id)}
    <HorizontalTabItem {tab} selected={tab.id === selectedId} />
  {/each}
  <button class="new-tab-btn" data-testid={side ? `new-tab-${side}` : "new-tab"} onclick={() => addTab(undefined, $activeWorkspaceId, side ?? undefined)} title={$t("tabs.create_new_tab_p0_8ca3", { p0: $keyboardShortcuts && shortcutLabel("new-tab") })} aria-label={$t("tabs.create_new_tab_400e")}>+</button>
</div>

<style>
  .tabs-scroll { display:flex; flex:0 1 auto; align-items:center; gap:4px; height:100%; min-width:0; max-width:100%; overflow-x:auto; overflow-y:hidden; scrollbar-width:none; }
  .tabs-scroll::-webkit-scrollbar { display:none; }
  .new-tab-btn { width:27px; height:27px; border-radius:var(--radius-sm); display:flex; align-items:center; justify-content:center; font-size:15px; font-weight:600; color:var(--text-secondary); background:rgba(255,255,255,.035); border:1px solid var(--border-subtle); flex-shrink:0; cursor:pointer; }
  .new-tab-btn:hover { background:var(--bg-glass-hover); color:var(--accent-primary); border-color:var(--accent-primary); }
</style>
