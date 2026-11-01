<script lang="ts">
  import { onDestroy } from "svelte";
  import { slide, fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import {
    tabs,
    addTab,
    selectTab,
    removeTab,
    suspendTab,
    reorderTabsList,
    changeTabWorkspace,
  } from "../../stores/browserStore";
  import { activeWorkspaceId, workspaces } from "../../stores/workspaceStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";

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

  function handleHibernateClick(event: MouseEvent, tabId: string) {
    event.stopPropagation();
    suspendTab(tabId);
  }

  let dragTabId = $state<string | null>(null);
  let isDragging = $state<boolean>(false);
  let dragStartY = 0;
  let hasMoved = false;

  function handleTabPointerDown(event: PointerEvent, tabId: string) {
    // Only primary left button or single touch
    if (event.button !== 0) return;

    // Do not initiate drag if user clicked an action button (close, sleep)
    const target = event.target as HTMLElement | null;
    if (target?.closest(".tab-action-btn")) {
      return;
    }

    dragTabId = tabId;
    dragStartY = event.clientY;
    hasMoved = false;
    isDragging = false;

    window.addEventListener("pointermove", handleWindowPointerMove);
    window.addEventListener("pointerup", handleWindowPointerUp);
    window.addEventListener("pointercancel", handleWindowPointerUp);
  }

  function handleWindowPointerMove(event: PointerEvent) {
    if (!dragTabId) return;

    const deltaY = event.clientY - dragStartY;
    if (!hasMoved && Math.abs(deltaY) > 4) {
      hasMoved = true;
      isDragging = true;
      document.body.style.cursor = "grabbing";
      document.body.style.userSelect = "none";
    }

    if (!isDragging) return;

    // Use elementFromPoint to find the element directly under cursor
    const element = document.elementFromPoint(event.clientX, event.clientY);
    if (!element) return;

    // Check workspace pill hover
    const wsPill = element.closest<HTMLElement>(".ws-pill");
    const allPills = document.querySelectorAll<HTMLElement>(".ws-pill");
    allPills.forEach((p) => p.classList.toggle("drop-hover", p === wsPill));

    // Check if hovering over another tab row to swap
    const targetRow = element.closest<HTMLElement>(".tab-row");
    if (targetRow) {
      const targetId = targetRow.getAttribute("data-tab-id");
      if (targetId && targetId !== dragTabId) {
        const currentIds = workspaceTabs.map((t) => t.id);
        const fromIdx = currentIds.indexOf(dragTabId);
        const toIdx = currentIds.indexOf(targetId);

        if (fromIdx !== -1 && toIdx !== -1) {
          const reordered = [...currentIds];
          const [moved] = reordered.splice(fromIdx, 1);
          reordered.splice(toIdx, 0, moved);
          reorderTabsList(reordered);
          dragStartY = event.clientY;
        }
      }
    }
  }

  function handleWindowPointerUp(event: PointerEvent) {
    window.removeEventListener("pointermove", handleWindowPointerMove);
    window.removeEventListener("pointerup", handleWindowPointerUp);
    window.removeEventListener("pointercancel", handleWindowPointerUp);

    document.body.style.cursor = "";
    document.body.style.userSelect = "";

    const allPills = document.querySelectorAll<HTMLElement>(".ws-pill");
    allPills.forEach((p) => p.classList.remove("drop-hover"));

    const wasDragging = isDragging;
    const clickedTabId = dragTabId;

    if (wasDragging && clickedTabId) {
      // Check if dropped onto a workspace pill
      const element = document.elementFromPoint(event.clientX, event.clientY);
      const wsPill = element?.closest<HTMLElement>(".ws-pill");
      const targetWsId = wsPill?.getAttribute("data-ws-id");
      if (targetWsId) {
        changeTabWorkspace(clickedTabId, targetWsId);
      }
    }

    dragTabId = null;
    isDragging = false;
    hasMoved = false;

    // Normal click selection if not dragged
    if (!wasDragging && clickedTabId) {
      selectTab(clickedTabId);
    }
  }

  function handleKeydown(event: KeyboardEvent, tabId: string) {
    if (event.key === "Enter") {
      selectTab(tabId);
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

<div class="tabs-section">
  <div class="tabs-header">
    <span class="section-label">
      {currentWorkspaceName.toUpperCase()} TABS ({workspaceTabs.length})
    </span>
    <button
      class="new-tab-pill"
      onclick={() => addTab("https://duckduckgo.com", $activeWorkspaceId)}
      title={`Create New Tab (${ $keyboardShortcuts && shortcutLabel("new-tab") })`}
    >
      <span>+ New</span>
      <kbd>{$keyboardShortcuts && shortcutLabel("new-tab")}</kbd>
    </button>
  </div>

  <div class="tabs-list">
    {#each workspaceTabs as tab (tab.id)}
      <div
        class="tab-row"
        data-tab-id={tab.id}
        class:active={tab.is_active}
        class:hibernated={tab.is_hibernated}
        class:is-dragging={dragTabId === tab.id && isDragging}
        onpointerdown={(e) => handleTabPointerDown(e, tab.id)}
        onkeydown={(e) => handleKeydown(e, tab.id)}
        role="button"
        tabindex="0"
        title="{tab.title} (Drag to reorder · {$keyboardShortcuts && shortcutLabel('move-tab-up')} / {$keyboardShortcuts && shortcutLabel('move-tab-down')})"
        transition:slide={{ duration: 220, easing: cubicOut }}
      >
        <div class="drag-grip" title="Drag to reorder">
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
        <span class="tab-glyph">{getTabGlyph(tab.url, tab.is_hibernated)}</span>
        <div class="tab-meta">
          <span class="tab-title">{tab.title}</span>
          <span class="tab-host">
            {getTabHost(tab.url)}{tab.is_hibernated ? " · Suspended" : ""}
          </span>
        </div>

        <div class="tab-actions">
          {#if !tab.is_active && !tab.is_hibernated}
            <button
              class="tab-action-btn sleep-btn"
              onclick={(e) => handleHibernateClick(e, tab.id)}
              title="Hibernate Tab to Free RAM"
              aria-label="Hibernate Tab"
            >
              💤
            </button>
          {/if}
          <button
            class="tab-action-btn close-btn"
            onclick={(e) => handleCloseClick(e, tab.id)}
            title={`Close Tab (${ $keyboardShortcuts && shortcutLabel("close-tab") })`}
            aria-label="Close Tab"
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
    transition:
      background 0.18s ease,
      transform 0.18s cubic-bezier(0.22, 1, 0.36, 1),
      box-shadow 0.18s ease;
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

  .tab-row.hibernated {
    opacity: 0.68;
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
