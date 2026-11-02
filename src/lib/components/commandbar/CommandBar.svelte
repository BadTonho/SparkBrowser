<script lang="ts">
  import { t } from "../../i18n";
  import { openSitePermissions } from "../../stores/sitePermissionsStore";
  import { setActiveSettingsCategory } from "../../stores/settingsStore";
  import { restoreClosedTab } from "../../stores/closedTabsStore";
  import { openTools } from "../../stores/toolsStore";
  import { rankCommands, normalizeSearch } from "../../services/command-ranking.js";
  import { focusDialog } from "../../actions/focusDialog";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import type { CommandItem } from "../../types/browser";
  import {
    isCommandBarOpen,
    commandQuery,
    commandBarScale,
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

  let inputEl: HTMLInputElement | undefined = $state();
  let selectedIndex = $state(0);

  $effect(() => {
    if ($isCommandBarOpen) {
      selectedIndex = 0;
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

    if (query.trim().length > 0) {
      items.push({
        id: "open-new-tab",
        title: $t("commandbar.open_p0_in_a_new_tab_1c56", { p0: query.trim() }),
        subtitle: $t("commandbar.creates_a_new_isolated_rust_webview_tab_23ab"),
        icon: "🚀",
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
        icon: "🧭",
        badge: $t("commandbar.navigate_5ea5"),
        action: async () => {
          const target = query.trim();
          await closeCommandBar();
          await submitNavigation(target);
        },
      });
    }

    // Matching open tabs
    for (const tab of $tabs) {
      if (
        !q ||
        normalizeSearch(tab.title).includes(q) ||
        normalizeSearch(tab.url).includes(q)
      ) {
        items.push({
          id: `switch-${tab.id}`,
          title: tab.title,
          subtitle: tab.url,
          icon: tab.is_hibernated ? "💤" : tab.is_active ? "🟢" : "🗂️",
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
        icon: "🧘",
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
        icon: "📥",
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
        icon: "⚙️",
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
        icon: "💤",
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
        icon: "🕒",
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
        icon: "🛡️",
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
        icon: "📐",
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
        icon: "🔄",
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

  async function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      await closeCommandBar();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      if (suggestions.length > 0) {
        selectedIndex = (selectedIndex + 1) % suggestions.length;
      }
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      if (suggestions.length > 0) {
        selectedIndex = (selectedIndex - 1 + suggestions.length) % suggestions.length;
      }
    } else if (event.key === "Enter") {
      event.preventDefault();
      const chosen = suggestions[selectedIndex] ?? suggestions[0];
      if (chosen) {
        await chosen.action();
      }
    }
  }
</script>

{#if $isCommandBarOpen}
  <div
    class="command-backdrop"
    transition:fade={{ duration: 150 }}
    onclick={() => closeCommandBar()}
    onkeydown={(e) => e.key === "Escape" && closeCommandBar()}
    role="presentation"
  >
    <div
      class="command-modal"
      style="transform: scale({$commandBarScale});"
      in:fly={{ y: -18, duration: 220, easing: cubicOut }}
      out:fade={{ duration: 130 }}
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      use:focusDialog={{ onEscape: closeCommandBar }}
      role="dialog"
      aria-modal="true"
      aria-label={$t("commandbar.command_bar_1356")}
      tabindex="-1"
    >
      <div class="command-input-bar">
        <span class="spark-icon">⚡</span>
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

      <div class="command-results">
        {#each suggestions as item, idx (item.id)}
          <button
            class="command-item"
            class:selected={idx === selectedIndex}
            onmouseenter={() => (selectedIndex = idx)}
            onclick={() => item.action()}
          >
            <span class="item-icon">{item.icon}</span>
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
  </div>
{/if}

<style>
  .command-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
    background: rgba(8, 8, 8, 0.62);
    backdrop-filter: blur(var(--overlay-backdrop-blur, 16px));
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 12vh;
  }

  .command-modal {
    width: min(620px, 92vw);
    border-radius: var(--radius-lg);
    background: rgba(20, 24, 38, 0.92);
    border: 1px solid var(--accent-primary);
    box-shadow:
      0 24px 64px rgba(0, 0, 0, 0.65),
      0 0 40px color-mix(in srgb, var(--accent-primary) 22%, transparent);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    will-change: transform;
  }

  .command-input-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 18px;
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(255, 255, 255, 0.02);
  }

  .spark-icon {
    font-size: 18px;
  }

  .command-input {
    flex: 1;
    font-size: 15px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .command-input::placeholder {
    color: var(--text-muted);
  }

  .esc-hint {
    font-size: 10.5px;
    padding: 3px 7px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
  }

  .command-results {
    display: flex;
    flex-direction: column;
    padding: 8px;
    gap: 4px;
    max-height: 340px;
    overflow-y: auto;
  }

  .command-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    text-align: left;
    transition:
      background 0.14s ease,
      transform 0.14s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .command-item.selected {
    background: linear-gradient(
      135deg,
      color-mix(in srgb, var(--accent-primary) 24%, transparent) 0%,
      color-mix(in srgb, var(--accent-secondary) 18%, transparent) 100%
    );
    border: 1px solid var(--accent-primary);
    transform: translateX(3px);
  }

  .item-icon {
    font-size: 16px;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .item-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .item-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-subtitle {
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-badge {
    font-size: 10.5px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .command-item.selected .item-badge {
    background: var(--accent-primary);
    color: var(--accent-foreground);
  }

  .command-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-top: 1px solid var(--border-subtle);
    background: rgba(0, 0, 0, 0.22);
    font-size: 11px;
    color: var(--text-muted);
  }

  .command-footer kbd {
    padding: 1px 5px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
    font-size: 10px;
  }
</style>
