<script lang="ts">
  import TabFavicon from "./TabFavicon.svelte";
  import ContainerBadge from "./ContainerBadge.svelte";
  import type { TabItem } from "../../types/browser";
  import { startTabDrag, tabDrag } from "../../stores/tabDragStore";
  import { removeTab, toggleTabPin, selectTab } from "../../stores/browserStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import { t } from "../../i18n";
  import { showTabContextMenu } from "../../stores/persistenceStore";
  import TabMuteButton from "../media/TabMuteButton.svelte";
  import { tabsMemory, refreshTabsMemory, formatTabMemory, formatTabSavings } from "../../stores/tabMemoryStore";
  import { tabs } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { clearTabSelection, handleTabSelectionInput, selectedTabIds } from "../../stores/tabSelectionStore";
  import { tabHoverCard, refreshVisiblePreviews } from "../../actions/tabHoverCard";
  let { tab, selected = false }: { tab: TabItem; selected?: boolean } = $props();
  let element: HTMLDivElement | undefined = $state();
  $effect(() => { if (selected) element?.scrollIntoView({ block: 'nearest', inline: 'nearest' }); });
  // Once the page of the selected tab has had time to draw, picture it for its hover card.
  $effect(() => {
    if (!selected) return;
    const timer = setTimeout(() => refreshVisiblePreviews(true), 1500);
    return () => clearTimeout(timer);
  });

  function getTabGlyph(rawUrl: string, isHibernated: boolean): string {
    if (isHibernated) return "sleep";
    const lower = rawUrl.toLowerCase();
    if (lower.includes("duckduckgo")) return "search";
    if (lower.includes("github")) return "code";
    if (lower.includes("rust-lang")) return "box";
    if (lower.includes("wikipedia")) return "globe";
    if (lower.includes("youtube")) return "play";
    return "✦";
  }

  function handleCloseClick(event: MouseEvent, tabId: string) {
    event.stopPropagation();
    removeTab(tabId);
  }

  function handlePointerDown(event: PointerEvent) {
    if (event.target instanceof Element && event.target.closest("button,input,a")) return;
    const orderedIds = $tabs.filter(item => !item.workspace_id || item.workspace_id === $activeWorkspaceId).map(item => item.id);
    if (handleTabSelectionInput(event, tab.id, orderedIds)) return;
    clearTabSelection();
    startTabDrag(event, tab.id);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.target instanceof Element && event.target.closest("button,input,a")) return;
    const orderedIds = $tabs.filter(item => !item.workspace_id || item.workspace_id === $activeWorkspaceId).map(item => item.id);
    if ((event.key === "Enter" || event.key === " ") && handleTabSelectionInput(event, tab.id, orderedIds)) return;
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      clearTabSelection();
      void selectTab(tab.id);
    }
  }

  function getTabMemoryLabel(tabItem: TabItem, memoryBytes?: number): string {
    if (tabItem.is_hibernated) return $t("tabs.sleeping_savings", { savings: formatTabSavings(tabItem.id) });
    return $t("tabs.memory_usage", { usage: formatTabMemory(memoryBytes, $t("tabs.memory_unavailable")) });
  }

</script>
      <div
        class="h-tab"
        bind:this={element}
        data-tab-id={tab.id}
        class:active={selected}
        class:multi-selected={$selectedTabIds.includes(tab.id)}
        class:hibernated={tab.is_hibernated}
        class:ephemeral={tab.is_ephemeral}
        class:is-dragging={$tabDrag?.id === tab.id && $tabDrag.dragging}
        onpointerdown={handlePointerDown}
        oncontextmenu={(event) => void showTabContextMenu(event, tab.id)}
        onmouseup={(e) => e.button === 1 && removeTab(tab.id)}
        onkeydown={handleKeydown}
        role="button"
        tabindex="0"
        aria-pressed={$selectedTabIds.includes(tab.id)}
        onpointerenter={() => refreshTabsMemory()}
        use:tabHoverCard={{ tabId: tab.id, title: tab.title, memory: getTabMemoryLabel(tab, $tabsMemory[tab.id]), noPreview: $t("tabs.hover_no_preview"), placement: "below" }}
      >
        <TabMuteButton tabId={tab.id} />
        <span class="tab-glyph"><TabFavicon url={tab.url} glyph={getTabGlyph(tab.url, tab.is_hibernated)} dimmed={tab.is_hibernated} /></span>
        {#if tab.container_id}<ContainerBadge id={tab.container_id} />{/if}
        {#if tab.is_ephemeral}
          <span class="ephemeral-badge" title={$t("tabs.ephemeral_tooltip")}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" width="12" height="12">
              <path d="M12 2a10 10 0 0 0-10 10c0 5.523 4.477 10 10 10s10-4.477 10-10a10 10 0 0 0-10-10z" />
              <path d="M12 12m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
            </svg>
          </span>
        {/if}
        <span class="tab-title" class:is-hibernated={tab.is_hibernated}>{tab.title}</span>
        {#if tab.is_hibernated}
          <span class="sleep-badge" title={$t("tabs.sleeping_savings", { savings: formatTabSavings(tab.id) })}>
            {formatTabSavings(tab.id)}
          </span>
        {/if}
        <button class="tab-action-btn tab-close-btn" aria-label={tab.is_pinned ? $t("tabs.unpin_tab_2055") : $t("tabs.pin_tab_bf50")} title={tab.is_pinned ? $t("tabs.unpin_tab_2055") : "Pin tab: protect from hibernation"} aria-pressed={!!tab.is_pinned} onclick={(event) => { event.stopPropagation(); void toggleTabPin(tab.id); }}>{tab.is_pinned ? "\u25C6" : "\u25C7"}</button>
        <button
          class="tab-close-btn"
          onclick={(e) => handleCloseClick(e, tab.id)}
          title={$t("tabs.close_tab_p0_d2d6", { p0: $keyboardShortcuts && shortcutLabel("close-tab") })}
          aria-label={$t("tabs.close_tab_50a3")}
        >
          ✕
        </button>
      </div>

<style>
  .h-tab {
    touch-action: none;
    position: relative;
    width: 240px;
    min-width: 120px;
    max-width: 240px;
    height: 28px;
    padding: 0 8px 0 10px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    gap: 7px;
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--border-subtle);
    cursor: grab;
    flex: 0 1 240px;
    transition:
      background 0.16s ease,
      border-color 0.16s ease,
      box-shadow 0.16s ease,
      transform 0.12s ease;
  }

  .h-tab:active {
    cursor: grabbing;
  }

  .h-tab:hover {
    background: var(--bg-glass-hover);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .h-tab.active {
    background: linear-gradient(
      135deg,
      color-mix(in srgb, var(--accent-primary) 22%, transparent) 0%,
      color-mix(in srgb, var(--accent-secondary) 14%, transparent) 100%
    );
    border-color: color-mix(in srgb, var(--accent-primary) 45%, transparent);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.25);
  }

  .h-tab.multi-selected { outline:1px solid var(--accent-primary); outline-offset:1px; }

  .h-tab.ephemeral {
    border-color: rgba(168, 85, 247, 0.35);
  }

  .h-tab.ephemeral.active {
    background: linear-gradient(
      135deg,
      rgba(168, 85, 247, 0.22) 0%,
      rgba(126, 34, 206, 0.14) 100%
    );
    border-color: rgba(168, 85, 247, 0.55);
  }

  .ephemeral-badge {
    color: #c084fc;
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }

  .h-tab.hibernated {
    opacity: 0.65;
    border-style: dashed;
    border-color: rgba(255, 255, 255, 0.08);
  }

  .h-tab.hibernated:hover {
    opacity: 0.95;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .tab-title.is-hibernated {
    color: var(--text-muted);
    font-style: italic;
  }

  .sleep-badge {
    font-size: 9.5px;
    font-weight: 500;
    color: #a78bfa;
    background: rgba(167, 139, 250, 0.12);
    border: 1px solid rgba(167, 139, 250, 0.25);
    padding: 1px 5px;
    border-radius: 4px;
    flex-shrink: 0;
    letter-spacing: 0.2px;
  }

  /* The tab itself follows the pointer as a copy; its slot stays as a quiet placeholder. */
  .h-tab.is-dragging {
    opacity: 0.35;
    border: 1px dashed var(--accent-primary);
    background: transparent;
    box-shadow: none;
    pointer-events: none;
  }

  .tab-glyph {
    font-size: 13px;
    flex-shrink: 0;
  }

  .tab-title {
    flex: 1;
    min-width: 0;
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-close-btn {
    width: 17px;
    height: 17px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    color: var(--text-muted);
    opacity: 0;
    flex-shrink: 0;
    transition: opacity 0.15s ease, background 0.15s ease, color 0.15s ease;
  }

  .h-tab:hover .tab-close-btn,
  .h-tab.active .tab-close-btn {
    opacity: 1;
  }

  .tab-close-btn:hover {
    background: rgba(232, 17, 35, 0.25);
    color: #ff6b78;
  }


  @media (prefers-reduced-motion: reduce) { .h-tab { transition: none; } }
</style>
