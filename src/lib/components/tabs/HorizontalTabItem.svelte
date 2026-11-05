<script lang="ts">
  import type { TabItem } from "../../types/browser";
  import { startTabDrag, tabDrag } from "../../stores/tabDragStore";
  import { removeTab, toggleTabPin, selectTab } from "../../stores/browserStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import { t } from "../../i18n";
  import TabMuteButton from "../media/TabMuteButton.svelte";
  import { tabsMemory, refreshTabsMemory, formatTabMemory, formatTabSavings } from "../../stores/tabMemoryStore";
  let { tab, selected = false }: { tab: TabItem; selected?: boolean } = $props();
  let element: HTMLDivElement | undefined = $state();
  $effect(() => { if (selected) element?.scrollIntoView({ block: 'nearest', inline: 'nearest' }); });
  function getTabHost(rawUrl: string): string {
    try {
      return new URL(rawUrl).hostname.replace(/^www\./, "");
    } catch {
      return rawUrl;
    }
  }

  function getTabGlyph(rawUrl: string, isHibernated: boolean): string {
    if (isHibernated) return "💤";
    const lower = rawUrl.toLowerCase();
    if (lower.includes("duckduckgo")) return "🦆";
    if (lower.includes("github")) return "🐙";
    if (lower.includes("rust-lang")) return "🦀";
    if (lower.includes("wikipedia")) return "🌐";
    if (lower.includes("youtube")) return "▶️";
    return "✦";
  }

  function handleCloseClick(event: MouseEvent, tabId: string) {
    event.stopPropagation();
    removeTab(tabId);
  }

  function getTabTooltip(tabItem: TabItem, memoryBytes?: number): string {
    const host = getTabHost(tabItem.url);
    if (tabItem.is_hibernated) {
      const savings = formatTabSavings(tabItem.id);
      return `${tabItem.title} — ${host}\n${$t("tabs.sleeping_savings", { savings })}`;
    }
    const memStr = formatTabMemory(memoryBytes, $t("tabs.memory_unavailable"));
    return `${tabItem.title} — ${host}\n${$t("tabs.memory_usage", { usage: memStr })}`;
  }

</script>
      <div
        class="h-tab"
        bind:this={element}
        data-tab-id={tab.id}
        class:active={selected}
        class:hibernated={tab.is_hibernated}
        class:ephemeral={tab.is_ephemeral}
        class:is-dragging={$tabDrag?.id === tab.id && $tabDrag.dragging}
        onpointerdown={(e) => startTabDrag(e, tab.id)}
        onmouseup={(e) => e.button === 1 && removeTab(tab.id)}
        onkeydown={(event) => { if (event.key === "Enter" || event.key === " ") { event.preventDefault(); void selectTab(tab.id); } }}
        role="button"
        tabindex="0"
        onpointerenter={() => refreshTabsMemory()}
        title={getTabTooltip(tab, $tabsMemory[tab.id])}
      >
        <TabMuteButton tabId={tab.id} />
        <span class="tab-glyph">{getTabGlyph(tab.url, tab.is_hibernated)}</span>
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
    width: 180px;
    min-width: 120px;
    max-width: 180px;
    height: 28px;
    padding: 0 8px 0 10px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    gap: 7px;
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--border-subtle);
    cursor: grab;
    flex: 0 1 180px;
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

  .h-tab.is-dragging {
    opacity: 0.85;
    border-color: var(--accent-primary);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    transform: scale(1.03);
    z-index: 30;
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
