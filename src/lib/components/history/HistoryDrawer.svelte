<script lang="ts">
  import { logIgnored } from "../../services/ignored-error";
  import AppIcon from "../icons/AppIcon.svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { t, formatDate } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { onMount, untrack } from "svelte";
  import { locale, type Locale } from "../../i18n";
  import { currentUrl, tabs } from "../../stores/browserStore";
  import { activeWorkspaceId } from "../../stores/workspaceSelectionStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { onHistoryOverlayAction, onHistoryTabAction, onHistoryTabLoaded, setHistoryOverlay, setHistoryTab, type HistoryOverlayAction, type HistoryOverlayPresentation } from "../../services/bridge/history-overlay";
  import { HISTORY_URL } from "../../stores/historyDisplayStore";
  import { describeError } from "../../services/user-errors.js";
  import { observePanelTheme, readGlassFrame } from "../panels/panel-host";
  import {
    historyEntries,
    historyBusy,
    historyError,
    historyHasMore,
    loadHistory,
    isHistoryDrawerOpen,
    closeHistoryDrawer,
    wipeBrowsingHistory,
  } from "../../stores/persistenceStore";
  import { addTab } from "../../stores/browserStore";
  import { ensureSiteIcon, siteIcons, siteKey } from "../../stores/siteIconStore";
  import { deleteHistoryEntries } from "../../services/bridge/persistence";
  import { historyRows } from "./history-rows";

  let filterQuery = $state("");
  const native = isTauriRuntime();
  let nativeSessionId = Date.now() * 1000;
  let nativeSessionKey = "";
  let lastNativePresentation = "";

  const activeTab = $derived($tabs.find((tab) => tab.is_active));
  /** The history tab being viewed (Settings → History display → Tab); it shows the same rows as the panel. */
  const historyTab = $derived(native ? $tabs.find((tab) => tab.is_active && tab.url === HISTORY_URL) : undefined);
  let tabSessionId = Date.now() * 1000;
  let tabSessionTab = "";
  let lastTabPresentation = "";

  let filteredHistory = $derived($historyEntries);
  // Each time a history tab comes into view, its list is read again from the start.
  $effect(() => {
    const id = historyTab?.id;
    if (!id) return;
    untrack(() => {
      if (id === tabSessionTab) return;
      tabSessionTab = id;
      tabSessionId += 1;
      lastTabPresentation = "";
      void loadHistory(filterQuery);
    });
  });
  $effect(() => {
    if (!$isHistoryDrawerOpen && !historyTab) return;
    const query = filterQuery;
    const timer = setTimeout(() => void loadHistory(query), 180);
    return () => clearTimeout(timer);
  });

  function formatTime(epochSecs: number): string {
    if (!epochSecs) return "";
    return $formatDate(epochSecs * 1000, { hour: "2-digit", minute: "2-digit" });
  }

  /** "Today - Friday, November 27, 2026", "Yesterday - …", or the date alone. */
  function dayLabel(epochSecs: number, relative: "today" | "yesterday" | null): string {
    const date = $formatDate(epochSecs * 1000, { weekday: "long", day: "numeric", month: "long", year: "numeric" });
    if (relative === "today") return $t("history.day_today", { date });
    if (relative === "yesterday") return $t("history.day_yesterday", { date });
    return date;
  }

  function nativeTheme(): HistoryOverlayPresentation["theme"] {
    const root = document.documentElement;
    const style = getComputedStyle(root);
    const light = root.dataset.theme === "light";
    return {
      surface: style.getPropertyValue("--bg-permission-popover").trim() || (light ? "#fff" : "#1c1c1c"),
      border: style.getPropertyValue("--border-subtle").trim() || (light ? "#0000001a" : "#ffffff17"),
      text: style.getPropertyValue("--text-primary").trim() || (light ? "#171717" : "#f5f5f5"),
      muted: style.getPropertyValue("--text-secondary").trim() || (light ? "#525252" : "#a3a3a3"),
      accent: style.getPropertyValue("--accent-primary").trim() || (light ? "#171717" : "#e5e5e5"),
      hover: style.getPropertyValue("--bg-glass-hover").trim() || (light ? "#0000000d" : "#ffffff14"),
      field: light ? "#f4f4f5" : "#242424",
      ...readGlassFrame(style),
    };
  }

  async function pushNativeOverlay() {
    if (!native) return;
    const tab = activeTab;
    if (!$isHistoryDrawerOpen || !tab) {
      if (nativeSessionKey) {
        const closingSession = nativeSessionId;
        nativeSessionKey = "";
        lastNativePresentation = "";
        await setHistoryOverlay(false, closingSession).catch(logIgnored);
      }
      return;
    }
    const key = `${$activeWorkspaceId}:${tab.id}:${tab.url}`;
    if (key !== nativeSessionKey) {
      if (nativeSessionKey) void setHistoryOverlay(false, nativeSessionId).catch(logIgnored);
      nativeSessionKey = key;
      nativeSessionId += 1;
      lastNativePresentation = "";
    }
    const payload = presentation(tab.id, tab.url || $currentUrl);
    const serialized = JSON.stringify(payload);
    if (serialized === lastNativePresentation) return;
    lastNativePresentation = serialized;
    await setHistoryOverlay(true, nativeSessionId, payload).catch((error) => {
      lastNativePresentation = "";
      historyError.set(describeError(error));
    });
  }

  /** The rows and texts of the history page, for the panel or the tab. */
  function presentation(tabId: string, url: string): HistoryOverlayPresentation {
    const rows = historyRows($historyEntries, Date.now(), siteKey);
    const icons: Record<string, string> = {};
    for (const row of rows) {
      ensureSiteIcon(row.url);
      const icon = row.site ? $siteIcons[row.site] : undefined;
      if (row.site && icon?.startsWith("data:image/") && icon.length <= 64 * 1024) icons[row.site] = icon;
    }
    return {
      tabId,
      url,
      locale: $locale as Locale,
      labels: {
        title: $t("history.browsing_history_sqlite_59d8"),
        clear: $t("history.clear_all_ddce"),
        close: $t("permissions.close_7d9e"),
        search: $t("history.search_visited_pages_9bc1"),
        empty: $t("history.no_history_entries_found_c307"),
        loadMore: $t("history.load_more_ac89"),
        selected: $t("history.selected_count"),
        open: $t("history.open_selected"),
        delete: $t("history.delete_selected"),
        cancelSelection: $t("history.cancel_selection"),
        moreFromSite: $t("history.more_from_site"),
        deleteEntry: $t("history.delete_entry"),
        menu: $t("history.entry_actions"),
        select: $t("history.select_entry"),
      },
      theme: nativeTheme(),
      query: filterQuery,
      entries: rows.map(({ visitedAt, relative, dayKey: _day, ...row }) => ({ ...row, time: formatTime(visitedAt), day: dayLabel(visitedAt, relative) })),
      icons,
      busy: $historyBusy,
      error: $historyError,
      hasMore: $historyHasMore,
    };
  }

  /** Sends the rows to the history tab being viewed. */
  async function pushHistoryTab(force = false) {
    const tab = historyTab;
    if (!tab) return;
    const payload = presentation(tab.id, HISTORY_URL);
    const serialized = JSON.stringify(payload);
    if (!force && serialized === lastTabPresentation) return;
    lastTabPresentation = serialized;
    await setHistoryTab(tab.id, tabSessionId, payload).catch((error) => {
      lastTabPresentation = "";
      // A page still loading is not a failure: "history-tab-loaded" sends the rows again.
      if (!String(error).includes("history_tab.not_ready")) historyError.set(describeError(error));
    });
  }

  /** Actions of the history tab: like the panel's, but the tab stays open. */
  async function handleTabAction(action: HistoryOverlayAction) {
    const tab = historyTab;
    if (!tab || action.tabId !== tab.id || action.sessionId !== tabSessionId) return;
    switch (action.action) {
      case "search": filterQuery = action.query; break;
      case "clear": await wipeBrowsingHistory(); break;
      case "load-more": await loadHistory(filterQuery, true); break;
      case "open":
        if (action.entryUrl) await addTab(action.entryUrl);
        break;
      case "open-many":
        for (const url of action.entryUrls) await addTab(url);
        break;
      case "more-from-site":
        if (action.entryHost) filterQuery = action.entryHost;
        break;
      case "delete":
        await deleteEntries(action.entryIds);
        break;
    }
  }

  async function handleNativeAction(action: import("../../services/bridge/history-overlay").HistoryOverlayAction) {
    const tab = activeTab;
    if (!nativeSessionKey || action.sessionId !== nativeSessionId || !tab || action.tabId !== tab.id || action.url !== (tab.url || $currentUrl) || !$isHistoryDrawerOpen) return;
    switch (action.action) {
      case "close": await closeHistoryDrawer(); break;
      case "search": filterQuery = action.query; break;
      case "clear": await wipeBrowsingHistory(); break;
      case "load-more": await loadHistory(filterQuery, true); break;
      case "open":
        if (action.entryUrl) await openHistoryUrl(action.entryUrl);
        break;
      case "open-many":
        await closeHistoryDrawer();
        for (const url of action.entryUrls) await addTab(url);
        break;
      case "more-from-site":
        if (action.entryHost) filterQuery = action.entryHost;
        break;
      case "delete":
        await deleteEntries(action.entryIds);
        break;
    }
  }

  $effect(() => {
    void $isHistoryDrawerOpen; void $historyEntries; void $historyBusy; void $historyError; void $historyHasMore;
    void filterQuery; void $currentUrl; void $tabs; void $activeWorkspaceId; void $locale; void $t; void $siteIcons;
    untrack(() => { void pushNativeOverlay(); });
  });
  $effect(() => {
    void historyTab; void $historyEntries; void $historyBusy; void $historyError; void $historyHasMore;
    void filterQuery; void $locale; void $t; void $siteIcons;
    untrack(() => { void pushHistoryTab(); });
  });

  onMount(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    const stopTheme = native ? observePanelTheme(() => {
      if (nativeSessionKey) void pushNativeOverlay().catch(logIgnored);
      if (historyTab) void pushHistoryTab(true).catch(logIgnored);
    }) : undefined;
    const stops: Array<() => void> = [];
    const keep = (unlisten: () => void) => { if (disposed) unlisten(); else stops.push(unlisten); };
    void onHistoryOverlayAction(handleNativeAction).then(keep);
    void onHistoryTabAction((action) => void handleTabAction(action)).then(keep);
    void onHistoryTabLoaded((tabId) => { if (historyTab?.id === tabId) void pushHistoryTab(true); }).then(keep);
    return () => {
      disposed = true;
      stopTheme?.();
      stop?.();
      for (const unlisten of stops.splice(0)) unlisten();
      if (nativeSessionKey) void setHistoryOverlay(false, nativeSessionId);
    };
  });

  /** Deletes visits and removes them from the list in place, so the pages already loaded stay. */
  async function deleteEntries(ids: number[]) {
    if (!ids.length) return;
    try {
      await deleteHistoryEntries(ids);
      const gone = new Set(ids);
      historyEntries.update((items) => items.filter((item) => !gone.has(item.id)));
    } catch (error) {
      historyError.set(describeError(error));
    }
  }

  async function openHistoryUrl(url: string) {
    await closeHistoryDrawer();
    await addTab(url);
  }
</script>

{#if $isHistoryDrawerOpen && !native}
  <div
    class="history-backdrop"
    transition:fade={{ duration: 150 }}
    onclick={() => closeHistoryDrawer()}
    onkeydown={(e) => e.key === "Escape" && closeHistoryDrawer()}
    role="presentation"
  >
    <div
      class="history-panel"
      in:fly={{ x: -28, duration: 220, easing: cubicOut }}
      out:fade={{ duration: 130 }}
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      use:focusDialog={{ onEscape: closeHistoryDrawer }}
      role="dialog"
      aria-modal="true"
      aria-label={$t("history.browsing_history_13cc")}
      tabindex="-1"
    >
      <div class="history-header">
        <div class="header-title">
          <span><AppIcon name="clock" /> {$t("history.browsing_history_sqlite_59d8")}</span>
        </div>
        <div class="header-actions">
          {#if $historyEntries.length > 0}
            <button class="clear-btn" onclick={() => void wipeBrowsingHistory()}>
              {$t("history.clear_all_ddce")}
            </button>
          {/if}
          <button class="close-btn" onclick={() => closeHistoryDrawer()}>
            ✕
          </button>
        </div>
      </div>

      <div class="history-search">
        <input
          type="text"
          bind:value={filterQuery}
          placeholder={$t("history.search_visited_pages_9bc1")}
          spellcheck="false"
        />
      </div>

      {#if $historyError}<ErrorMessage error={$historyError} />{/if}
      <div class="history-list" aria-busy={$historyBusy}>
        {#if filteredHistory.length === 0}
          <div class="empty-state">
            <p>{$t("history.no_history_entries_found_c307")}</p>
          </div>
        {:else}
          {#each filteredHistory as item (item.id)}
            <button
              class="history-row"
              onclick={() => openHistoryUrl(item.url)}
            >
              <div class="history-meta">
                <span class="history-title">{item.title}</span>
                <span class="history-url">{item.url}</span>
              </div>
              <span class="history-time">{formatTime(item.visited_at)}</span>
            </button>
          {/each}
        {/if}
        {#if $historyHasMore}<button disabled={$historyBusy} onclick={() => loadHistory(filterQuery, true)}>{$t("history.load_more_ac89")}</button>{/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .history-backdrop {
    position: fixed;
    inset: 0;
    z-index: 990;
    background: transparent;
    display: flex;
    justify-content: flex-start;
    padding: 46px 0 12px 12px;
  }

  .history-panel {
    width: min(420px, 90vw);
    height: 100%;
    border-radius: var(--radius-lg);
    background: var(--modal-surface-bg, var(--bg-permission-popover, #1c1c1c));
    backdrop-filter: var(--modal-surface-filter);
    -webkit-backdrop-filter: var(--modal-surface-filter);
    border: var(--modal-surface-border, 1px solid color-mix(in srgb, var(--accent-primary) 35%, transparent));
    box-shadow: var(--modal-surface-shadow, 0 20px 56px rgba(0, 0, 0, 0.6));
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .history-header {
    padding: 14px 16px;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-title {
    font-size: 13.5px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .clear-btn {
    padding: 4px 9px;
    border-radius: 6px;
    background: rgba(232, 17, 35, 0.16);
    color: #ff7b86;
    font-size: 11px;
    font-weight: 600;
  }

  .clear-btn:hover {
    background: rgba(232, 17, 35, 0.28);
  }

  .close-btn {
    width: 26px;
    height: 26px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .close-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
  }

  .history-search {
    padding: 10px 14px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .history-search input {
    width: 100%;
    padding: 8px 11px;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-subtle);
    font-size: 12.5px;
    color: var(--text-primary);
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .empty-state {
    padding: 36px 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12.5px;
  }

  .history-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 9px 11px;
    border-radius: var(--radius-md);
    text-align: left;
    transition: background 0.15s ease, transform 0.15s ease;
  }

  .history-row:hover {
    background: var(--bg-glass-hover);
    transform: translateX(2px);
  }

  .history-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .history-title {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .history-url {
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .history-time {
    font-size: 10.5px;
    color: var(--text-muted);
    flex-shrink: 0;
  }
</style>
