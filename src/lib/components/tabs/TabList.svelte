<script lang="ts">
  import { createListFlip } from "../../services/list-flip";
  import { showTabContextMenu, openTabGroupPopover } from "../../stores/persistenceStore";
  import { tabDividers, moveTabDivider } from "../../stores/tabDividersStore";
  import TabDivider from "./TabDivider.svelte";
  import TabGroupHeader from "./TabGroupHeader.svelte";
  import { tabGroups, loadTabGroups, removeTabsFromGroup } from "../../stores/tabGroupsStore";
  
  import AppIcon from "../icons/AppIcon.svelte";
  import TabFavicon from "./TabFavicon.svelte";
  import ContainerBadge from "./ContainerBadge.svelte";
  import { startTabDrag, cancelTabDrag, tabDrag } from "../../stores/tabDragStore";
  import TabMuteButton from "../media/TabMuteButton.svelte";
  import { t } from "../../i18n";
  import { onDestroy, onMount, tick } from "svelte";
  import { slide, fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import {
    tabs,
    addTab,
    addEphemeralTab,
    selectTab,
    removeTab,
    toggleTabPin,
    suspendTab,
  } from "../../stores/browserStore";
  import { activeWorkspaceId, workspaces } from "../../stores/workspaceStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import { tabsMemory, refreshTabsMemory, formatTabMemory, formatTabSavings } from "../../stores/tabMemoryStore";
  import { clearTabSelection, handleTabSelectionInput, selectedTabIds } from "../../stores/tabSelectionStore";
  import { splitLayout } from "../../stores/splitViewStore";
  import { splitDragPreview } from "../../stores/tabDragStore";
  import { enterSplitView, exitSplitView } from "../../stores/splitViewActions";

  type SplitPane = "left" | "right" | "other";
  const PANES: SplitPane[] = ["left", "right", "other"];

  let listEl = $state<HTMLDivElement>();
  const flip = createListFlip(() => listEl, ".tab-row, .tab-divider:not(.dragging)");

  let workspaceTabs = $derived(
    $tabs.filter(
      (t) => !t.workspace_id || t.workspace_id === $activeWorkspaceId
    )
  );
  /** The split being shown in this workspace, if any. */
  let activeSplit = $derived($splitLayout && $splitLayout.workspace_id === $activeWorkspaceId ? $splitLayout : null);
  /** Which side of the split a tab belongs to; "other" for tabs that are not in the split. */
  const paneOf = (tabId: string): SplitPane => {
    // A tab being dragged to the other side is already shown there, so the drop place is visible.
    if (activeSplit && $splitDragPreview?.id === tabId) return $splitDragPreview.side;
    if (activeSplit?.left_tab_ids.includes(tabId)) return "left";
    if (activeSplit?.right_tab_ids.includes(tabId)) return "right";
    return "other";
  };
  /** Tabs in the order they are shown: the tabs of a group are gathered where its first tab is. */
  let groupedTabs = $derived.by(() => {
    const members = $tabGroups.members;
    const present = new Map<string, typeof workspaceTabs>();
    for (const tab of workspaceTabs) {
      const group = members[tab.id];
      if (group) present.set(group, [...(present.get(group) ?? []), tab]);
    }
    const shown = new Set<string>();
    return workspaceTabs.flatMap((tab) => {
      const group = members[tab.id];
      if (!group) return [tab];
      if (shown.has(group)) return [];
      shown.add(group);
      return present.get(group) ?? [];
    });
  });
  /** While split, the list is gathered by side (left, right, then the rest); otherwise it is unchanged. */
  let displayTabs = $derived.by(() => {
    if (!activeSplit) return groupedTabs;
    const list = PANES.flatMap((pane) => groupedTabs.filter((tab) => paneOf(tab.id) === pane));
    const preview = $splitDragPreview;
    const dragged = preview && list.find((tab) => tab.id === preview.id);
    if (!preview || !dragged) return list;
    // Where the dragged tab would land: before the tab it is over, or at the end of that side.
    const rest = list.filter((tab) => tab !== dragged);
    let index = preview.beforeId ? rest.findIndex((tab) => tab.id === preview.beforeId) : -1;
    if (index < 0) {
      const lastOnSide = rest.map((tab) => paneOf(tab.id)).lastIndexOf(preview.side);
      index = lastOnSide >= 0 ? lastOnSide + 1 : rest.findIndex((tab) => paneOf(tab.id) === "other");
    }
    rest.splice(index < 0 ? rest.length : index, 0, dragged);
    return rest;
  });
  /** A plain line before the right side and before the tabs outside the split, placed just before the first
   *  tab of each (same slot as a group header). The left side, at the top, needs none. */
  let paneSeparators = $derived(
    activeSplit
      ? PANES.filter((pane) => pane !== "left").flatMap((pane) => {
          const first = displayTabs.findIndex((tab) => paneOf(tab.id) === pane);
          return first >= 0 ? [{ pane, order: 4 * (first + 1) - 2 }] : [];
        })
      : [],
  );
  /** One entry per group that has a tab in this workspace, placed just before its first tab. */
  let visibleGroups = $derived(
    $tabGroups.groups.flatMap((group) => {
      const first = displayTabs.findIndex((tab) => $tabGroups.members[tab.id] === group.id);
      const count = displayTabs.filter((tab) => $tabGroups.members[tab.id] === group.id).length;
      return first >= 0 ? [{ group, count, order: 4 * (first + 1) - 2 }] : [];
    }),
  );
  /** A tab of a collapsed group is hidden, unless it is the one being viewed. */
  const isHidden = (tab: (typeof workspaceTabs)[number]) => {
    const group = $tabGroups.groups.find((item) => item.id === $tabGroups.members[tab.id]);
    if (!group) return false;
    // A side-list group keeps all of its tabs in the list; a folded one keeps only the tab being viewed.
    return group.mode === "popup" || (group.collapsed && !tab.is_active);
  };
  const groupOf = (tabId: string) => $tabGroups.groups.find((item) => item.id === $tabGroups.members[tabId]);
  onMount(() => { void loadTabGroups(); });
  let visibleDividers = $derived(
    $tabDividers.flatMap((divider) => {
      const index = divider.after === "" ? -1 : displayTabs.findIndex((tab) => tab.id === divider.after);
      return divider.after === "" || index >= 0 ? [{ divider, order: 4 * (index + 1) + 1 }] : [];
    }),
  );
  /** Right-click on empty space: the same menu as a tab, applied to the end of the list. */
  function handleListContextMenu(event: MouseEvent) {
    const last = workspaceTabs.at(-1);
    if (!last) { event.preventDefault(); return; }
    void showTabContextMenu(event, last.id, true);
  }
  /** The tab a divider dragged to height `y` belongs below: the last tab whose middle is above the pointer.
   *  Uses layout positions, so rows that are still sliding into place do not disturb the answer. */
  function tabAfterPointer(y: number): string {
    if (!listEl) return "";
    const top = listEl.getBoundingClientRect().top - listEl.scrollTop;
    let after = "", lowest = -1;
    for (const row of listEl.querySelectorAll<HTMLElement>(".tab-row")) {
      const middle = row.offsetTop - listEl.offsetTop + row.offsetHeight / 2;
      if (top + middle < y && middle > lowest) { after = row.dataset.tabId ?? after; lowest = middle; }
    }
    return after;
  }
  /** Moves a divider and slides the tabs (and other dividers) around it into their new places. */
  async function moveDivider(id: string, after: string) {
    flip.measure();
    moveTabDivider(id, after);
    await tick();
    flip.play();
  }
  const order = $derived(displayTabs.map(tab => tab.id).join("|") + "#" + $tabGroups.groups.map(group => group.collapsed ? group.id : "").join("") + $tabGroups.groups.map(group => group.mode).join(""));
  $effect.pre(() => { order; flip.measure(); });
  $effect(() => { order; flip.play(); });


  let currentWorkspaceName = $derived(
    $workspaces.find((w) => w.id === $activeWorkspaceId)?.name ?? "Workspace"
  );
  let activeTabId = $derived(workspaceTabs.find((tab) => tab.is_active)?.id);

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
    removeTabsFromGroup([tabId]);
    removeTab(tabId);
  }

  function getTabTooltip(tabItem: any, memoryBytes?: number): string {
    const memLabel = tabItem.is_hibernated
      ? $t("tabs.sleeping_savings", { savings: formatTabSavings(tabItem.id) })
      : $t("tabs.memory_usage", { usage: formatTabMemory(memoryBytes, $t("tabs.memory_unavailable")) });
    const reorderHint = `${$t("tabs.drag_to_reorder_f64b")} ${$keyboardShortcuts && shortcutLabel('move-tab-up')} / ${$keyboardShortcuts && shortcutLabel('move-tab-down')})`;
    return `${tabItem.title} ${reorderHint}\n${memLabel}`;
  }

  function handleHibernateClick(event: MouseEvent, tabId: string) {
    event.stopPropagation();
    suspendTab(tabId);
  }

  onDestroy(cancelTabDrag);
  function handleKeydown(event: KeyboardEvent, tabId: string) {
    if (event.target instanceof Element && event.target.closest("button,input,a")) return;
    if ((event.key === "Enter" || event.key === " ") && handleTabSelectionInput(event, tabId, workspaceTabs.map((tab) => tab.id))) return;
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      clearTabSelection();
      selectTab(tabId);
    }
  }

  function handlePointerDown(event: PointerEvent, tabId: string) {
    if (event.target instanceof Element && event.target.closest("button,input,a")) return;
    if (handleTabSelectionInput(event, tabId, workspaceTabs.map((tab) => tab.id))) return;
    startTabDrag(event, tabId);
  }

  $effect(() => {
    const id = activeTabId;
    if (!id || !listEl) return;
    requestAnimationFrame(() => {
      const row = [...(listEl?.querySelectorAll<HTMLElement>("[data-tab-id]") ?? [])].find((element) => element.dataset.tabId === id);
      row?.scrollIntoView({ block: "nearest" });
    });
  });

</script>

<div class="tabs-section" role="presentation" oncontextmenu={handleListContextMenu}>
  <div class="tabs-header">
    <span class="section-label">
      {currentWorkspaceName.toUpperCase()} {$t("tabs.tabs_d820")}{workspaceTabs.length})
    </span>
    <div class="tabs-header-actions">
      {#if !activeSplit}
        <button class="split-toggle" data-testid="split-enter" onclick={() => void enterSplitView()} title={$t("tabs.split_enter")} aria-label={$t("tabs.split_enter")}>◫</button>
      {/if}
      <button
        class="new-tab-pill ephemeral"
        onclick={() => addEphemeralTab()}
        title={$t("tabs.new_ephemeral") + " · " + $t("tabs.ephemeral_tooltip")}
      >
        <span>{$t("tabs.ephemeral_badge")}</span>
      </button>
      <button
        class="new-tab-pill"
        onclick={() => addTab(undefined, $activeWorkspaceId)}
        title={$t("tabs.create_new_tab_p0_8ca3", { p0: $keyboardShortcuts && shortcutLabel("new-tab") })}
      >
        <span>{$t("tabs.new_af6a")}</span>
        <kbd>{$keyboardShortcuts && shortcutLabel("new-tab")}</kbd>
      </button>
    </div>
  </div>

  <div class="tabs-list" bind:this={listEl}>
    <!-- Rendered before the group headers: on the same slot, the separator then stays above the header. -->
    {#each paneSeparators as item (item.pane)}
      <div class="pane-separator" style:order={item.order} data-pane-side={item.pane === "other" ? undefined : item.pane} data-testid={`split-separator-${item.pane}`} role="separator" aria-label={$t(`tabs.split_pane_${item.pane}`)} transition:slide={{ duration: 160, easing: cubicOut }}>
        <span class="line"></span>
        {#if item.pane === "right"}
          <button class="split-toggle" data-testid="split-exit" onpointerdown={(event) => event.stopPropagation()} onclick={() => void exitSplitView()} title={$t("tabs.split_exit")} aria-label={$t("tabs.split_exit")}>✕</button>
        {/if}
      </div>
    {/each}
    {#each visibleGroups as item (item.group.id)}
      <TabGroupHeader group={item.group} order={item.order} count={item.count} onopen={openTabGroupPopover} />
    {/each}
    {#each displayTabs as tab, index (tab.id)}
      {#if !isHidden(tab)}
      <div
        class="tab-row"
        class:in-group={!!groupOf(tab.id)}
        data-pane-side={activeSplit && paneOf(tab.id) !== "other" ? paneOf(tab.id) : undefined}
        data-group-id={groupOf(tab.id)?.id}
        data-color={groupOf(tab.id)?.color}
        style:order={4 * (index + 1)}
        data-tab-id={tab.id}
        class:active={tab.is_active}
        class:multi-selected={$selectedTabIds.includes(tab.id)}
        class:hibernated={tab.is_hibernated}
        class:ephemeral={tab.is_ephemeral}
        class:is-dragging={$tabDrag?.id === tab.id && $tabDrag.dragging}
        onpointerdown={(e) => handlePointerDown(e, tab.id)}
        oncontextmenu={(event) => void showTabContextMenu(event, tab.id)}
        onkeydown={(e) => handleKeydown(e, tab.id)}
        role="button"
        tabindex="0"
        aria-pressed={$selectedTabIds.includes(tab.id)}
        onpointerenter={() => refreshTabsMemory()}
        title={getTabTooltip(tab, $tabsMemory[tab.id])}
        transition:slide={{ duration: 220, easing: cubicOut }}
      >
        <div class="drag-grip" title={$t("tabs.drag_to_reorder_ef86")}>
          <svg width="7" height="13" viewBox="0 0 8 14" fill="currentColor">
            <circle cx="2" cy="2" r="1.3" />
            <circle cx="6" cy="2" r="1.3" />
            <circle cx="2" cy="7" r="1.3" />
            <circle cx="6" cy="7" r="1.3" />
            <circle cx="2" cy="12" r="1.3" />
            <circle cx="6" cy="12" r="1.3" />
          </svg>
        </div>
        <div class="tab-active-bar"></div>
        <TabMuteButton tabId={tab.id} />
        <span class="tab-glyph"><TabFavicon url={tab.url} glyph={getTabGlyph(tab.url, tab.is_hibernated)} dimmed={tab.is_hibernated} /></span>
        {#if tab.container_id}<ContainerBadge id={tab.container_id} />{/if}
        <div class="tab-meta">
          <span class="tab-title" class:is-hibernated={tab.is_hibernated}>{tab.title}</span>
          {#if tab.is_hibernated || tab.is_ephemeral}
            <span class="tab-host">
              {#if tab.is_hibernated}
                <span class="hibernated-badge"><AppIcon name="sleep" /> {formatTabSavings(tab.id)}</span>
              {/if}
              {#if tab.is_ephemeral}
                <span class="ephemeral-tag"><AppIcon name="eye-off" /> {$t("tabs.ephemeral_badge")}</span>
              {/if}
            </span>
          {/if}
        </div>

        <div class="tab-actions">
          <button class="tab-action-btn tab-close-btn" aria-label={tab.is_pinned ? $t("tabs.unpin_tab_2055") : $t("tabs.pin_tab_bf50")} title={tab.is_pinned ? $t("tabs.unpin_tab_2055") : "Pin tab: protect from hibernation"} aria-pressed={!!tab.is_pinned} onclick={(event) => { event.stopPropagation(); void toggleTabPin(tab.id); }}>{tab.is_pinned ? "\u25C6" : "\u25C7"}</button>
          {#if !tab.is_active && !tab.is_hibernated}
            <button
              class="tab-action-btn sleep-btn"
              onclick={(e) => handleHibernateClick(e, tab.id)}
              title={$t("tabs.hibernate_tab_to_free_ram_8298")}
              aria-label={$t("tabs.hibernate_tab_1b5f")}
            >
              <AppIcon name="sleep" />
            </button>
          {/if}
          <button
            class="tab-action-btn close-btn"
            onclick={(e) => handleCloseClick(e, tab.id)}
            title={$t("tabs.close_tab_p0_14f2", { p0: $keyboardShortcuts && shortcutLabel("close-tab") })}
            aria-label={$t("tabs.close_tab_f271")}
            transition:fade={{ duration: 120 }}
          >
            ✕
          </button>
        </div>
      </div>
      {/if}
    {/each}
    <!-- Dividers are placed with CSS `order` instead of living between the rows: moving one to another
         slot then never moves its element in the DOM, so a drag in progress is not interrupted. -->
    {#each visibleDividers as item (item.divider.id)}
      <TabDivider divider={item.divider} order={item.order} locate={tabAfterPointer} move={moveDivider} />
    {/each}
  </div>
</div>

<style>
  .tabs-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    min-height: 0;
  }

  .tabs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px;
  }

  .section-label {
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.8px;
    color: var(--text-muted);
  }

  .tabs-header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .new-tab-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--accent-primary) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent-primary) 30%, transparent);
    color: var(--text-primary);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background 0.18s ease,
      transform 0.18s cubic-bezier(0.22, 1, 0.36, 1),
      box-shadow 0.18s ease;
  }

  .new-tab-pill.ephemeral {
    background: rgba(168, 85, 247, 0.12);
    border-color: rgba(168, 85, 247, 0.35);
    color: #d8b4fe;
  }

  .new-tab-pill.ephemeral:hover {
    background: rgba(168, 85, 247, 0.22);
    box-shadow: 0 4px 12px rgba(168, 85, 247, 0.2);
  }

  .new-tab-pill:hover {
    background: color-mix(in srgb, var(--accent-primary) 26%, transparent);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px color-mix(in srgb, var(--accent-primary) 22%, transparent);
  }

  .new-tab-pill kbd {
    font-size: 9.5px;
    padding: 1px 4px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.3);
    color: var(--text-secondary);
    font-family: inherit;
  }

  .tabs-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow-y: auto;
    padding-right: 2px;
  }

  .pane-separator {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 6px 6px 4px;
    padding: 2px 4px;
    user-select: none;
  }
  .pane-separator .line { flex: 1; height: 1px; background: color-mix(in srgb, var(--accent-primary) 45%, transparent); }

  .split-toggle {
    flex: none;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  }
  .split-toggle:hover {
    background: color-mix(in srgb, var(--accent-primary) 22%, transparent);
    border-color: color-mix(in srgb, var(--accent-primary) 45%, transparent);
    color: var(--text-primary);
  }

  .drag-grip {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    opacity: 0;
    cursor: grab;
    flex-shrink: 0;
    transition: opacity 0.15s ease, color 0.15s ease;
    margin-right: -4px;
    margin-left: -2px;
  }

  .tab-row:hover .drag-grip {
    opacity: 0.65;
  }

  .tab-row:hover .drag-grip:hover {
    opacity: 1;
    color: var(--text-primary);
  }

  .tab-row {
    touch-action: none;
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px 8px 8px;
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid transparent;
    cursor: grab;
    user-select: none;
    touch-action: none;
    transition:
      background 0.18s ease,
      border-color 0.18s ease,
      opacity 0.2s ease,
      transform 0.18s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .tab-row:active {
    cursor: grabbing;
  }

  /* The row itself follows the pointer as a copy; its slot stays as a quiet placeholder. */
  .tab-row.is-dragging {
    opacity: 0.35;
    background: transparent;
    border: 1px dashed var(--accent-primary);
    box-shadow: none;
    pointer-events: none;
  }

  .tab-row.ephemeral {
    border-color: rgba(168, 85, 247, 0.25);
  }

  .tab-row.multi-selected { outline:1px solid var(--accent-primary); background:color-mix(in srgb,var(--accent-primary) 12%,var(--bg-glass-hover)); }

  .tab-row.ephemeral.active {
    background: linear-gradient(
      135deg,
      rgba(168, 85, 247, 0.22) 0%,
      rgba(126, 34, 206, 0.14) 100%
    );
    border-color: rgba(168, 85, 247, 0.45);
  }

  .ephemeral-tag {
    color: #c084fc;
    font-size: 10px;
    font-weight: 500;
  }

  .tab-row.hibernated {
    opacity: 0.65;
    border: 1px dashed rgba(255, 255, 255, 0.08);
  }

  .tab-row.hibernated:hover {
    opacity: 0.95;
    border-color: rgba(255, 255, 255, 0.18);
  }

  .tab-title.is-hibernated {
    color: var(--text-muted);
    font-style: italic;
  }

  .hibernated-badge {
    color: #a78bfa;
    font-size: 10px;
    font-weight: 500;
  }

  .tab-row:hover {
    background: var(--bg-glass-hover);
    border-color: var(--border-subtle);
    opacity: 1;
    transform: translateX(2px);
  }

  .tab-row.active {
    background: linear-gradient(
      135deg,
      color-mix(in srgb, var(--accent-primary) 20%, transparent) 0%,
      color-mix(in srgb, var(--accent-secondary) 14%, transparent) 100%
    );
    border-color: color-mix(in srgb, var(--accent-primary) 42%, transparent);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.22);
    opacity: 1;
  }

  .tab-active-bar {
    width: 3px;
    height: 18px;
    border-radius: 99px;
    background: transparent;
    transition: background 0.2s ease, height 0.2s ease;
  }

  .tab-row.active .tab-active-bar {
    background: var(--accent-gradient);
    height: 24px;
  }

  .tab-row.in-group {
    --group-color: #9aa0a6;
    margin-left: 10px;
    border-left: 3px solid var(--group-color);
    border-top-left-radius: 4px;
    border-bottom-left-radius: 4px;
  }
  .tab-row[data-color="grey"] { --group-color: #9aa0a6; }
  .tab-row[data-color="blue"] { --group-color: #5b9cf5; }
  .tab-row[data-color="red"] { --group-color: #f26b6b; }
  .tab-row[data-color="yellow"] { --group-color: #f2c14e; }
  .tab-row[data-color="green"] { --group-color: #4cc38a; }
  .tab-row[data-color="pink"] { --group-color: #f472b6; }
  .tab-row[data-color="purple"] { --group-color: #a78bfa; }
  .tab-row[data-color="cyan"] { --group-color: #4cc9f0; }

  .tab-glyph {
    font-size: 14px;
    flex-shrink: 0;
  }

  .tab-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .tab-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-host {
    font-size: 10.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .tab-action-btn {
    width: 20px;
    height: 20px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: var(--text-muted);
    opacity: 0;
    transition:
      opacity 0.15s ease,
      background 0.15s ease,
      color 0.15s ease,
      transform 0.15s ease;
  }

  .tab-row:hover .tab-action-btn,
  .tab-row.active .close-btn {
    opacity: 1;
  }

  .sleep-btn:hover {
    background: color-mix(in srgb, var(--accent-primary) 22%, transparent);
    transform: scale(1.08);
  }

  .close-btn:hover {
    background: rgba(232, 17, 35, 0.22);
    color: #ff6b78;
    transform: scale(1.08);
  }
</style>
