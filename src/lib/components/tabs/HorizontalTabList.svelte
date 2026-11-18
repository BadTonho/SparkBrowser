<script lang="ts">
  import type { TabItem } from "../../types/browser";
  import { addTab, addEphemeralTab } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceStore";
  import { t } from "../../i18n";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import HorizontalTabItem from "./HorizontalTabItem.svelte";
  import { createListFlip } from "../../services/list-flip";
  let { items, selectedId, side = null }: { items: TabItem[]; selectedId: string; side?: "left" | "right" | null } = $props();
  let list = $state<HTMLElement>();
  const flip = createListFlip(() => list, ".h-tab");
  const order = $derived(items.map(tab => tab.id).join("|"));
  $effect.pre(() => { order; flip.measure(); });
  $effect(() => { order; flip.play(); });
</script>

<div class="tabs-scroll" bind:this={list} data-pane-side={side ?? undefined} onwheel={(event) => { event.currentTarget.scrollLeft += event.deltaY; }} data-tauri-drag-region aria-label={side ? $t(side === "left" ? "tabs.left_pane_tabs" : "tabs.right_pane_tabs") : undefined}>
  {#each items as tab (tab.id)}
    <HorizontalTabItem {tab} selected={tab.id === selectedId} />
  {/each}
  <button class="new-tab-btn" data-testid={side ? `new-tab-${side}` : "new-tab"} onclick={() => addTab(undefined, $activeWorkspaceId, side ?? undefined)} title={$t("tabs.create_new_tab_p0_8ca3", { p0: $keyboardShortcuts && shortcutLabel("new-tab") })} aria-label={$t("tabs.create_new_tab_400e")}>+</button>
  <button class="new-tab-btn ephemeral-btn" onclick={() => addEphemeralTab()} title={$t("tabs.new_ephemeral") + " · " + $t("tabs.ephemeral_tooltip")} aria-label={$t("tabs.new_ephemeral")}>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13">
      <path d="M12 2a10 10 0 0 0-10 10c0 5.523 4.477 10 10 10s10-4.477 10-10a10 10 0 0 0-10-10z" />
      <path d="M12 12m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
    </svg>
  </button>
</div>

<style>
  .tabs-scroll { display:flex; flex:0 1 auto; align-items:center; gap:4px; height:100%; min-width:0; max-width:100%; overflow-x:auto; overflow-y:hidden; scrollbar-width:none; }
  .tabs-scroll::-webkit-scrollbar { display:none; }
  .new-tab-btn { width:27px; height:27px; border-radius:var(--radius-sm); display:flex; align-items:center; justify-content:center; font-size:15px; font-weight:600; color:var(--text-secondary); background:rgba(255,255,255,.035); border:1px solid var(--border-subtle); flex-shrink:0; cursor:pointer; }
  .new-tab-btn:hover { background:var(--bg-glass-hover); color:var(--accent-primary); border-color:var(--accent-primary); }
  .new-tab-btn.ephemeral-btn { color: rgba(168, 85, 247, 0.7); }
  .new-tab-btn.ephemeral-btn:hover { color: #c084fc; border-color: #a855f7; background: rgba(168, 85, 247, 0.12); }
</style>
