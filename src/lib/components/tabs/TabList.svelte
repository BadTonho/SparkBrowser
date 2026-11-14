<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import TabFavicon from "./TabFavicon.svelte";
  import ContainerBadge from "./ContainerBadge.svelte";
  import { startTabDrag, cancelTabDrag, tabDrag } from "../../stores/tabDragStore";
  import TabMuteButton from "../media/TabMuteButton.svelte";
  import { t } from "../../i18n";
  import { onDestroy } from "svelte";
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

  let listEl = $state<HTMLDivElement>();

  let workspaceTabs = $derived(
    $tabs.filter(
      (t) => !t.workspace_id || t.workspace_id === $activeWorkspaceId
    )
  );

  let currentWorkspaceName = $derived(
    $workspaces.find((w) => w.id === $activeWorkspaceId)?.name ?? "Workspace"
  );

  function getTabHost(rawUrl: string): string {
    try {
      return new URL(rawUrl).hostname.replace(/^www\./, "");
    } catch {
      return rawUrl;
    }
  }

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
    const activeTab = workspaceTabs.find((tab) => tab.is_active);
    if (!activeTab || !listEl) return;
    requestAnimationFrame(() => {
      const row = [...(listEl?.querySelectorAll<HTMLElement>("[data-tab-id]") ?? [])].find((element) => element.dataset.tabId === activeTab.id);
      row?.scrollIntoView({ block: "nearest" });
    });
  });

</script>

<div class="tabs-section">
  <div class="tabs-header">
    <span class="section-label">
      {currentWorkspaceName.toUpperCase()} {$t("tabs.tabs_d820")}{workspaceTabs.length})
    </span>
    <div class="tabs-header-actions">
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
    {#each workspaceTabs as tab (tab.id)}
      <div
        class="tab-row"
        data-tab-id={tab.id}
        class:active={tab.is_active}
        class:multi-selected={$selectedTabIds.includes(tab.id)}
        class:hibernated={tab.is_hibernated}
        class:ephemeral={tab.is_ephemeral}
        class:is-dragging={$tabDrag?.id === tab.id && $tabDrag.dragging}
        onpointerdown={(e) => handlePointerDown(e, tab.id)}
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
          <span class="tab-host">
            {getTabHost(tab.url)}
            {#if tab.is_hibernated}
              <span class="hibernated-badge">· <AppIcon name="sleep" /> {formatTabSavings(tab.id)}</span>
            {/if}
            {#if tab.is_ephemeral}
              <span class="ephemeral-tag">· <AppIcon name="eye-off" /> {$t("tabs.ephemeral_badge")}</span>
            {/if}
          </span>
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

  .tab-row.is-dragging {
    opacity: 0.88;
    background: color-mix(in srgb, var(--accent-primary) 24%, var(--bg-glass-hover));
    border: 1px solid var(--accent-primary);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    transform: scale(1.02);
    z-index: 50;
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
