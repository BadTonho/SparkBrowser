<script lang="ts">
  import { onDestroy } from "svelte";
  import {
    tabs,
    addTab,
    selectTab,
    removeTab,
    reorderTabsList,
  } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import {
    minimizeWindow,
    toggleMaximizeWindow,
    closeWindow,
  } from "../../services/tauri-bridge";

  let workspaceTabs = $derived(
    $tabs.filter(
      (t) => !t.workspace_id || t.workspace_id === $activeWorkspaceId
    )
  );

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

  function handleWheel(event: WheelEvent) {
    const target = event.currentTarget as HTMLElement;
    if (target) {
      target.scrollLeft += event.deltaY;
    }
  }

  let dragTabId = $state<string | null>(null);
  let isDragging = $state<boolean>(false);
  let dragStartX = 0;
  let hasMoved = false;

  function handleTabPointerDown(event: PointerEvent, tabId: string) {
    if (event.button !== 0) return;
    const target = event.target as HTMLElement | null;
    if (target?.closest(".tab-close-btn")) return;

    dragTabId = tabId;
    dragStartX = event.clientX;
    hasMoved = false;
    isDragging = false;

    window.addEventListener("pointermove", handleWindowPointerMove);
    window.addEventListener("pointerup", handleWindowPointerUp);
    window.addEventListener("pointercancel", handleWindowPointerUp);
  }

  function handleWindowPointerMove(event: PointerEvent) {
    if (!dragTabId) return;

    const deltaX = event.clientX - dragStartX;
    if (!hasMoved && Math.abs(deltaX) > 4) {
      hasMoved = true;
      isDragging = true;
      document.body.style.cursor = "grabbing";
      document.body.style.userSelect = "none";
    }

    if (!isDragging) return;

    const element = document.elementFromPoint(event.clientX, event.clientY);
    if (!element) return;

    const targetTab = element.closest<HTMLElement>(".h-tab");
    if (targetTab) {
      const targetId = targetTab.getAttribute("data-tab-id");
      if (targetId && targetId !== dragTabId) {
        const currentIds = workspaceTabs.map((t) => t.id);
        const fromIdx = currentIds.indexOf(dragTabId);
        const toIdx = currentIds.indexOf(targetId);

        if (fromIdx !== -1 && toIdx !== -1) {
          const reordered = [...currentIds];
          const [moved] = reordered.splice(fromIdx, 1);
          reordered.splice(toIdx, 0, moved);
          reorderTabsList(reordered);
          dragStartX = event.clientX;
        }
      }
    }
  }

  function handleWindowPointerUp() {
    window.removeEventListener("pointermove", handleWindowPointerMove);
    window.removeEventListener("pointerup", handleWindowPointerUp);
    window.removeEventListener("pointercancel", handleWindowPointerUp);

    document.body.style.cursor = "";
    document.body.style.userSelect = "";

    const wasDragging = isDragging;
    const clickedTabId = dragTabId;

    dragTabId = null;
    isDragging = false;
    hasMoved = false;

    if (!wasDragging && clickedTabId) {
      selectTab(clickedTabId);
    }
  }

  onDestroy(() => {
    window.removeEventListener("pointermove", handleWindowPointerMove);
    window.removeEventListener("pointerup", handleWindowPointerUp);
    window.removeEventListener("pointercancel", handleWindowPointerUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  });
</script>

<div
  class="h-tabbar"
  data-tauri-drag-region
>
  <div class="tabs-scroll" onwheel={handleWheel} data-tauri-drag-region>
    {#each workspaceTabs as tab (tab.id)}
      <div
        class="h-tab"
        data-tab-id={tab.id}
        class:active={tab.is_active}
        class:hibernated={tab.is_hibernated}
        class:is-dragging={dragTabId === tab.id && isDragging}
        onpointerdown={(e) => handleTabPointerDown(e, tab.id)}
        onmouseup={(e) => e.button === 1 && removeTab(tab.id)}
        role="button"
        tabindex="0"
        title="{tab.title} — {getTabHost(tab.url)}"
      >
        <span class="tab-glyph">{getTabGlyph(tab.url, tab.is_hibernated)}</span>
        <span class="tab-title">{tab.title}</span>
        <button
          class="tab-close-btn"
          onclick={(e) => handleCloseClick(e, tab.id)}
          title={`Close tab (${ $keyboardShortcuts && shortcutLabel("close-tab") })`}
          aria-label="Close tab"
        >
          ✕
        </button>
      </div>
    {/each}

    <button
      class="new-tab-btn"
      onclick={() => addTab("https://duckduckgo.com", $activeWorkspaceId)}
      title={`Create New Tab (${ $keyboardShortcuts && shortcutLabel("new-tab") })`}
      aria-label="Create New Tab"
    >
      +
    </button>
  </div>

  <div class="drag-spacer" data-tauri-drag-region></div>

  <div class="window-controls">
    <button class="win-btn" onclick={minimizeWindow} title="Minimize" aria-label="Minimize">
      <svg width="12" height="12" viewBox="0 0 12 12"><rect x="2" y="5.5" width="8" height="1" fill="currentColor" /></svg>
    </button>
    <button class="win-btn" onclick={toggleMaximizeWindow} title="Maximize" aria-label="Maximize">
      <svg width="12" height="12" viewBox="0 0 12 12"><rect x="2.5" y="2.5" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.2" fill="none" /></svg>
    </button>
    <button class="win-btn close-btn" onclick={closeWindow} title="Close" aria-label="Close">
      <svg width="12" height="12" viewBox="0 0 12 12"><path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" /></svg>
    </button>
  </div>
</div>

<style>
  .h-tabbar {
    height: 38px;
    width: 100%;
    display: flex;
    align-items: center;
    background: var(--bg-glass-primary);
    border-bottom: 1px solid var(--border-subtle);
    backdrop-filter: blur(20px);
    z-index: 18;
    padding: 0 4px 0 10px;
    user-select: none;
    gap: 8px;
  }

  .tabs-scroll {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    max-width: fit-content;
  }

  .tabs-scroll::-webkit-scrollbar {
    display: none;
  }

  .drag-spacer {
    flex: 1;
    height: 100%;
    min-width: 20px;
  }

  .h-tab {
    position: relative;
    min-width: 120px;
    max-width: 210px;
    height: 28px;
    padding: 0 8px 0 10px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    gap: 7px;
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--border-subtle);
    cursor: grab;
    flex-shrink: 0;
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

  .h-tab.hibernated {
    opacity: 0.7;
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

  .new-tab-btn {
    width: 27px;
    height: 27px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--border-subtle);
    flex-shrink: 0;
    cursor: pointer;
    transition:
      background 0.16s ease,
      color 0.16s ease,
      border-color 0.16s ease,
      transform 0.14s ease;
  }

  .new-tab-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--accent-primary);
    border-color: var(--accent-primary);
    transform: scale(1.05);
  }

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
</style>
