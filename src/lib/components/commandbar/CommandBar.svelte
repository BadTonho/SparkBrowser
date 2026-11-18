<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { createCommandQuery } from "../../services/command-query.js";
  import { searchCommandItems, emptyCommandSearch } from "../../services/bridge/command-search";
  import type { CommandSearchResult } from "../../types/command-search";
  import { openSavedNote, openSavedSession } from "../../stores/toolsContentStore";
  import { describeError } from "../../services/user-errors.js";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { t } from "../../i18n";
  import { openSitePermissions } from "../../stores/sitePermissionsStore";
  import { setActiveSettingsCategory } from "../../stores/settingsStore";
  import { restoreClosedTab } from "../../stores/closedTabsStore";
  import { openTools } from "../../stores/toolsStore";
  import { rankCommands, normalizeSearch } from "../../services/command-ranking.js";
  import { focusDialog } from "../../actions/focusDialog";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { mountPanelHost } from "../panels/panel-host";
  import { onCommandsPopoverAction, setCommandsPopover } from "../../services/bridge/commands-popover";
  import "./commandbar.css";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import type { CommandItem } from "../../types/browser";
  import {
    isCommandBarOpen,
    commandQuery,
    commandBarScale,
    commandBarSession,
    beginCommandBarSession,
    endCommandBarSession,
    closeCommandBar,
  } from "../../stores/commandBarStore";
  import {
    tabs,
    addTab,
    selectTab,
    submitNavigation,
    suspendInactiveTabs,
    toggleSidebar,
    triggerReload,
  } from "../../stores/browserStore";
  import { bookmarks, openHistoryDrawer } from "../../stores/persistenceStore";
  import { flipAdblockShield } from "../../stores/privacyStore";
  import { workspaces, activateWorkspace } from "../../stores/workspaceStore";
  import { openDownloadsDrawer } from "../../stores/downloadsStore";
  import {
    openSettingsModal,
    toggleZenMode,
    tabLayout,
  } from "../../stores/settingsStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import { locale } from "../../i18n";
  import { calculateQuery } from "../../services/calculator.js";

  // In the app the bar is drawn by a native layer above the page (so the page stays visible behind it);
  // the real bar below keeps all the logic, off screen, and only a copy of the markup is shown there.
  const native = isTauriRuntime();
  $effect(() => {
    if (!native) return;
    if ($isCommandBarOpen && !$commandBarSession) {
      const anchor = document.querySelector(".browser-root");
      if (anchor instanceof HTMLElement) beginCommandBarSession(anchor);
    } else if (!$isCommandBarOpen && $commandBarSession) {
      endCommandBarSession();
    }
  });
  function host(node: HTMLElement) {
    if (!native) return {};
    const stop = mountPanelHost(node, true, {
      id: "commands",
      sessionStore: commandBarSession,
      close: async () => { await closeCommandBar(); },
      set: setCommandsPopover,
      listen: onCommandsPopoverAction,
    });
    return { destroy: stop };
  }
  // Outside the app the bar keeps its own motion and focus handling.
  const fadeOutside = (node: Element, params: { duration: number }) => native ? { duration: 0 } : fade(node, params);
  const flyOutside = (node: Element, params: { y: number; duration: number; easing: (t: number) => number }) => native ? { duration: 0 } : fly(node, params);
  function dialog(node: HTMLElement) {
    return native ? {} : focusDialog(node, { onEscape: closeCommandBar });
  }
  const present = $derived(native ? Boolean($commandBarSession) : $isCommandBarOpen);

  let inputEl: HTMLInputElement | undefined = $state();
  let selectedId = $state<string | null>(null);
  let searchResults = $state<CommandSearchResult>(emptyCommandSearch);
  let searchError = $state("");
  let actionError = $state("");
  let executing = $state(false);
  const queryRunner = createCommandQuery({ search: searchCommandItems,
    result: (result: CommandSearchResult) => searchResults = result,
    error: (error: unknown) => searchError = describeError(error) });
  $effect(() => {
    const query = $commandQuery; const open = $isCommandBarOpen;
    selectedId = null; searchResults = emptyCommandSearch; searchError = ""; actionError = "";
    queryRunner.set(open ? query : "");
    return () => queryRunner.stop();
  });
  async function execute(item: CommandItem) {
    if (executing) return;
    executing = true;
    searchError = "";
    try { await item.action(); } catch (e) { searchError = describeError(e); }
    finally { executing = false; }
  }

  $effect(() => {
    if ($isCommandBarOpen) {
      selectedId = null;
      setTimeout(() => {
        inputEl?.focus();
        inputEl?.select();
      }, 20);
    }
  });

  function buildSuggestions(query: string): CommandItem[] {
    const shortcuts = $keyboardShortcuts;
    const q = normalizeSearch(query.trim());
    const items: CommandItem[] = [];

    const calculation = calculateQuery(query, $locale);
    if (calculation) {
      const item: CommandItem = calculation.error
        ? {
            id: "calculator-result", title: $t("calculator.error_title"),
            subtitle: $t(calculation.error), icon: "=", badge: $t("calculator.calculator"),
            action: async () => {},
          }
        : {
            id: "calculator-result", title: $t("calculator.result", { value: calculation.text }),
            subtitle: $t("calculator.copy_result"), icon: "=", badge: $t("calculator.calculator"),
            action: async () => {
              try {
                await navigator.clipboard.writeText(calculation.text);
                await closeCommandBar();
              } catch {
                actionError = $t("calculator.copy_failed");
              }
            },
          };
      return [item];
    }

    if (query.trim().length > 0) {
      items.push({
        id: "open-new-tab",
        title: $t("commandbar.open_p0_in_a_new_tab_1c56", { p0: query.trim() }),
        subtitle: $t("commandbar.creates_a_new_isolated_rust_webview_tab_23ab"),
        icon: "rocket",
        badge: $t("commandbar.new_tab_822c"),
        action: async () => {
          const target = query.trim();
          await closeCommandBar();
          await addTab(target);
        },
      });

      items.push({
        id: "navigate-current",
        title: $t("commandbar.navigate_current_tab_to_p0_62a5", { p0: query.trim() }),
        subtitle: $t("commandbar.loads_url_or_search_in_the_active_tab_822c"),
        icon: "compass",
        badge: $t("commandbar.navigate_5ea5"),
        action: async () => {
          const target = query.trim();
          await closeCommandBar();
          await submitNavigation(target);
        },
      });
    }

    for (const note of searchResults.notes) items.push({
      id: `note-${note.url}`, title: note.title || note.url, subtitle: note.preview || note.url,
      icon: "✎", badge: $t("search.notes"),
      action: async () => { await openSavedNote(note.url); await closeCommandBar(); },
    });
    for (const session of searchResults.sessions) items.push({
      id: `session-${session.id}`, title: session.name, subtitle: session.preview,
      icon: "▤", badge: $t("search.session_pages", { count: session.page_count }),
      action: async () => { await openSavedSession(session.id); await closeCommandBar(); },
    });

    // Matching open tabs
    for (const tab of $tabs) {
      if (
        !q ||
        normalizeSearch(tab.title).includes(q) ||
        normalizeSearch(tab.url).includes(q)
      ) {
        const workspaceName = $workspaces.find((workspace) => workspace.id === tab.workspace_id)?.name ?? tab.workspace_id;
        items.push({
          id: `switch-${tab.id}`,
          title: tab.title,
          subtitle: `${tab.url} · ${$t("commandbar.tab_in_workspace", { p0: workspaceName })}`,
          icon: tab.is_hibernated ? "sleep" : tab.is_active ? "status-online" : "tabs",
          badge: tab.is_active
            ? $t("common.active_tab")
            : tab.is_hibernated
              ? $t("common.wake_tab")
              : $t("common.switch_tab"),
          action: async () => {
            await closeCommandBar();
            await selectTab(tab.id);
          },
        });
      }
    }

    // Matching Workspaces
    for (const ws of $workspaces) {
      if (!q || normalizeSearch(ws.name).includes(q) || "workspace".includes(q)) {
        items.push({
          id: `ws-${ws.id}`,
          title: $t("commandbar.switch_to_p0_workspace_c856", { p0: ws.name }),
          subtitle: $t("commandbar.applies_p0_dynamic_theme_and_tab_context_3fb6", { p0: ws.name }),
          icon: ws.icon,
          badge: $t("commandbar.workspace_87bb"),
          action: async () => {
            await closeCommandBar();
            await activateWorkspace(ws.id);
          },
        });
      }
    }

    // Matching SQLite Bookmarks
    for (const bm of $bookmarks) {
      if (!q || normalizeSearch(bm.title).includes(q) || normalizeSearch(bm.url).includes(q)) {
        items.push({
          id: `bm-${bm.id}`,
          title: $t("commandbar.bookmark_p0_5d38", { p0: bm.title }),
          subtitle: bm.url,
          icon: bm.icon,
          badge: $t("commandbar.bookmark_1bac"),
          action: async () => {
            await closeCommandBar();
            await addTab(bm.url);
          },
        });
      }
    }

    const browserCommands: CommandItem[] = [
      { id: "cmd-tab-diagnostics", title: $t("diagnostics.heading"), subtitle: $t("diagnostics.description"), icon: "◉", badge: $t("commandbar.tools_ea93"), action: async () => { await closeCommandBar(); await openTools("diagnostics"); } },
      { id: "cmd-reopen-tab", title: $t("closed.reopen"), subtitle: $t("closed.reopen_description"), icon: "↶", badge: shortcutLabel("reopen-tab"), action: async () => { await closeCommandBar(); await restoreClosedTab(); } },
      { id: "cmd-closed-tabs", title: $t("closed.heading"), subtitle: $t("closed.description"), icon: "↶", badge: $t("commandbar.tools_ea93"), action: async () => { await closeCommandBar(); await openTools("closed"); } },
      { id: "cmd-site-permissions", title: $t("commandbar.open_site_permissions_a835"), subtitle: $t("commandbar.manage_permissions_for_the_focused_page_adbc"), icon: "\u26BF", badge: $t("commandbar.privacy_54a5"), action: async () => { await closeCommandBar(); await openSitePermissions(); } },
      { id: "cmd-site-search", title: $t("commandbar.configure_site_search_shortcuts_a867"), subtitle: $t("commandbar.add_keyword_searches_5c01"), icon: "\u2315", badge: $t("commandbar.search_49c2"), action: async () => { await closeCommandBar(); setActiveSettingsCategory("search_navigation"); await openSettingsModal(); } },
      { id: "cmd-split-view", title: $t("commandbar.configure_split_view_348c"), subtitle: $t("commandbar.choose_two_workspace_tabs_962a"), icon: "\u25EB", badge: $t("commandbar.tabs_8e5e"), action: async () => { await closeCommandBar(); await openTools("tabs"); } },
      ...(["sessions", "tabs", "notes"] as const).map(section => ({ id: `cmd-tools-${section}`, title: $t(({ sessions: "common.open_sessions", tabs: "common.open_tabs", notes: "common.open_notes" } as const)[section]), subtitle: $t("commandbar.browser_tools_321e"), icon: "\u2637", badge: $t("commandbar.tools_ea93"), action: async () => { await closeCommandBar(); await openTools(section); } })),
      {
        id: "cmd-zen",
        title: $t("commandbar.toggle_zen_focus_mode_eaad"),
        subtitle: $t("commandbar.hide_all_browser_toolbars_for_100_fullscreen_web_p0_ae51", { p0: shortcuts && shortcutLabel("zen") }),
        icon: "leaf",
        badge: $t("commandbar.zen_mode_826b"),
        action: async () => {
          await closeCommandBar();
          toggleZenMode();
        },
      },
      {
        id: "cmd-downloads",
        title: $t("commandbar.open_download_manager_d563"),
        subtitle: $t("commandbar.view_downloaded_files_or_save_a_url_p0_4a0b", { p0: shortcutLabel("downloads") }),
        icon: "download",
        badge: $t("commandbar.downloads_d5fd"),
        action: async () => {
          await closeCommandBar();
          await openDownloadsDrawer();
        },
      },
      {
        id: "cmd-settings",
        title: $t("commandbar.open_browser_settings_shortcuts_55cd"),
        subtitle: $t("commandbar.customize_themes_hibernation_timer_view_shortcuts_p0_4bdc", { p0: shortcutLabel("settings") }),
        icon: "settings",
        badge: $t("commandbar.settings_74a8"),
        action: async () => {
          await closeCommandBar();
          await openSettingsModal();
        },
      },
      {
        id: "cmd-hibernate",
        title: $t("commandbar.hibernate_inactive_tabs_free_ram_5e90"),
        subtitle: $t("commandbar.suspends_background_webviews_while_keeping_tabs_in_sidebar_16ea"),
        icon: "sleep",
        badge: $t("commandbar.ram_saver_c859"),
        action: async () => {
          await closeCommandBar();
          await suspendInactiveTabs(0);
        },
      },
      {
        id: "cmd-history",
        title: $t("commandbar.open_browsing_history_a45d"),
        subtitle: $t("commandbar.search_or_clear_local_sqlite_history_p0_148b", { p0: shortcutLabel("history") }),
        icon: "clock",
        badge: "SQLite",
        action: async () => {
          await closeCommandBar();
          await openHistoryDrawer();
        },
      },
      {
        id: "cmd-adblock",
        title: $t("commandbar.toggle_native_rust_adblock_shield_a0eb"),
        subtitle: $t("commandbar.enable_or_pause_network_cosmetic_tracker_blocking_1652"),
        icon: "shield",
        badge: $t("commandbar.privacy_54a5"),
        action: async () => {
          await closeCommandBar();
          await flipAdblockShield();
        },
      },
      {
        id: "cmd-toggle-sidebar",
        title: $t("commandbar.toggle_sidebar_1d4c"),
        subtitle: $t("commandbar.collapse_or_expand_the_vertical_tab_bar_p0_b3fc", { p0: shortcutLabel("sidebar") }),
        icon: "ruler",
        badge: $t("commandbar.command_7131"),
        action: async () => {
          await closeCommandBar();
          toggleSidebar();
        },
      },
      {
        id: "cmd-reload",
        title: $t("commandbar.reload_active_tab_89d3"),
        subtitle: $t("commandbar.refresh_the_current_web_page_p0_9ea5", { p0: shortcutLabel("reload-tab") }),
        icon: "refresh",
        badge: $t("commandbar.command_7131"),
        action: async () => {
          await closeCommandBar();
          await triggerReload();
        },
      },
    ];

    for (const cmd of browserCommands) {
      if (cmd.id === "cmd-toggle-sidebar" && $tabLayout !== "vertical") {
        continue;
      }
      if (!q || normalizeSearch(cmd.title).includes(q) || normalizeSearch(cmd.subtitle).includes(q)) {
        items.push(cmd);
      }
    }

    return rankCommands(items, query);
  }

  let suggestions = $derived($isCommandBarOpen ? buildSuggestions($commandQuery) : []);
  const selectedIndex = $derived(Math.max(0, suggestions.findIndex(item => item.id === selectedId)));

  async function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      await closeCommandBar();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      if (suggestions.length > 0) {
        selectedId = suggestions[(selectedIndex + 1) % suggestions.length].id;
      }
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      if (suggestions.length > 0) {
        selectedId = suggestions[(selectedIndex - 1 + suggestions.length) % suggestions.length].id;
      }
    } else if (event.key === "Enter") {
      event.preventDefault();
      const chosen = suggestions[selectedIndex] ?? suggestions[0];
      if (chosen) {
        await execute(chosen);
      }
    }
  }
</script>

{#snippet modal()}
  <div
    class="command-modal"
    style={native ? undefined : `transform: scale(${$commandBarScale});`}
    in:flyOutside={{ y: -18, duration: 220, easing: cubicOut }}
    out:fadeOutside={{ duration: 130 }}
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    use:dialog
    use:host
    role="dialog"
    aria-modal="true"
    aria-label={$t("commandbar.command_bar_1356")}
    tabindex="-1"
  >
    <div class="command-input-bar">
      <span class="spark-icon"><AppIcon name="zap" /></span>
      <input
        bind:this={inputEl}
        bind:value={$commandQuery}
        onkeydown={handleKeydown}
        class="command-input"
        type="text"
        placeholder={$t("commandbar.type_url_search_web_switch_workspace_tab_or_run_command_0026")}
        spellcheck="false"
      />
      <kbd class="esc-hint">ESC</kbd>
    </div>

    <ErrorMessage error={searchError} />
    {#if actionError}<p class="action-error" role="alert">{actionError}</p>{/if}
    {#if searchResults.notes_count || searchResults.sessions_count}<p class="search-counts" role="status">{$t("search.counts", { notes: `${searchResults.notes_count}${searchResults.notes_has_more ? "+" : ""}`, sessions: `${searchResults.sessions_count}${searchResults.sessions_has_more ? "+" : ""}` })}</p>{/if}
    <div class="command-results">
      {#each suggestions as item, idx (item.id)}
        <button
          class="command-item"
          data-command-id={item.id}
          disabled={executing}
          class:selected={idx === selectedIndex}
          onmouseenter={() => (selectedId = item.id)}
          onclick={() => execute(item)}
        >
          <span class="item-icon"><AppIcon name={item.icon} /></span>
          <div class="item-text">
            <span class="item-title">{item.title}</span>
            <span class="item-subtitle">{item.subtitle}</span>
          </div>
          <span class="item-badge">{item.badge}</span>
        </button>
      {/each}
    </div>

    <div class="command-footer">
      <span><kbd>↑</kbd> <kbd>↓</kbd> {$t("commandbar.to_navigate_5a70")}</span>
      <span><kbd>↵</kbd> {$t("commandbar.to_execute_5a2d")}</span>
      <span><kbd>{$keyboardShortcuts && shortcutLabel("command-bar")}</kbd> {$t("commandbar.anytime_75eb")}</span>
    </div>
  </div>
{/snippet}

{#if present}
  {#if native}
    <!-- The real bar stays off screen; only the copy inside it is shown in the native layer. -->
    <div class="command-controller" aria-hidden="true">{@render modal()}</div>
  {:else}
    <div
      class="command-backdrop"
      transition:fade={{ duration: 150 }}
      onclick={() => closeCommandBar()}
      onkeydown={(e) => e.key === "Escape" && closeCommandBar()}
      role="presentation"
    >
      {@render modal()}
    </div>
  {/if}
{/if}
