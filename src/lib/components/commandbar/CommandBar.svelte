<script lang="ts">
  import { openSitePermissions } from "../../stores/sitePermissionsStore";
  import { setActiveSettingsCategory } from "../../stores/settingsStore";
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
        title: `Open "${query.trim()}" in a New Tab`,
        subtitle: "Creates a new isolated Rust WebView tab",
        icon: "🚀",
        badge: "New Tab ↵",
        action: async () => {
          const target = query.trim();
          await closeCommandBar();
          await addTab(target);
        },
      });

      items.push({
        id: "navigate-current",
        title: `Navigate current tab to "${query.trim()}"`,
        subtitle: "Loads URL or search in the active tab",
        icon: "🧭",
        badge: "Navigate",
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
            ? "Active Tab"
            : tab.is_hibernated
              ? "Wake Tab"
              : "Switch Tab",
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
          title: `Switch to ${ws.name} Workspace`,
          subtitle: `Applies ${ws.name} dynamic theme and tab context`,
          icon: ws.icon,
          badge: "Workspace",
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
          title: `Bookmark: ${bm.title}`,
          subtitle: bm.url,
          icon: bm.icon,
          badge: "Bookmark",
          action: async () => {
            await closeCommandBar();
            await addTab(bm.url);
          },
        });
      }
    }

    const browserCommands: CommandItem[] = [
      { id: "cmd-site-permissions", title: "Open Site Permissions", subtitle: "Manage permissions for the focused page", icon: "\u26BF", badge: "Privacy", action: async () => { await closeCommandBar(); await openSitePermissions(); } },
      { id: "cmd-site-search", title: "Configure Site Search Shortcuts", subtitle: "Add keyword searches", icon: "\u2315", badge: "Search", action: async () => { await closeCommandBar(); setActiveSettingsCategory("search_navigation"); await openSettingsModal(); } },
      { id: "cmd-split-view", title: "Configure Split View", subtitle: "Choose two workspace tabs", icon: "\u25EB", badge: "Tabs", action: async () => { await closeCommandBar(); await openTools("tabs"); } },
      ...(["sessions", "tabs", "notes"] as const).map(section => ({ id: `cmd-tools-${section}`, title: `Open ${section}`, subtitle: "Browser tools", icon: "\u2637", badge: "Tools", action: async () => { await closeCommandBar(); await openTools(section); } })),
      {
        id: "cmd-zen",
        title: "Toggle Zen Focus Mode",
        subtitle: `Hide all browser toolbars for 100% fullscreen web (${shortcuts && shortcutLabel("zen")})`,
        icon: "🧘",
        badge: "Zen Mode",
        action: async () => {
          await closeCommandBar();
          toggleZenMode();
        },
      },
      {
        id: "cmd-downloads",
        title: "Open Download Manager",
        subtitle: `View downloaded files or save a URL (${shortcutLabel("downloads")})`,
        icon: "📥",
        badge: "Downloads",
        action: async () => {
          await closeCommandBar();
          await openDownloadsDrawer();
        },
      },
      {
        id: "cmd-settings",
        title: "Open Browser Settings & Shortcuts",
        subtitle: `Customize themes, hibernation timer & view shortcuts (${shortcutLabel("settings")})`,
        icon: "⚙️",
        badge: "Settings",
        action: async () => {
          await closeCommandBar();
          await openSettingsModal();
        },
      },
      {
        id: "cmd-hibernate",
        title: "Hibernate Inactive Tabs (Free RAM)",
        subtitle: "Suspends background WebViews while keeping tabs in sidebar",
        icon: "💤",
        badge: "RAM Saver",
        action: async () => {
          await closeCommandBar();
          await suspendInactiveTabs(0);
        },
      },
      {
        id: "cmd-history",
        title: "Open Browsing History",
        subtitle: `Search or clear local SQLite history (${shortcutLabel("history")})`,
        icon: "🕒",
        badge: "SQLite",
        action: async () => {
          await closeCommandBar();
          await openHistoryDrawer();
        },
      },
      {
        id: "cmd-adblock",
        title: "Toggle Native Rust AdBlock Shield",
        subtitle: "Enable or pause network & cosmetic tracker blocking",
        icon: "🛡️",
        badge: "Privacy",
        action: async () => {
          await closeCommandBar();
          await flipAdblockShield();
        },
      },
      {
        id: "cmd-toggle-sidebar",
        title: "Toggle Sidebar",
        subtitle: `Collapse or expand the vertical tab bar (${shortcutLabel("sidebar")})`,
        icon: "📐",
        badge: "Command",
        action: async () => {
          await closeCommandBar();
          toggleSidebar();
        },
      },
      {
        id: "cmd-reload",
        title: "Reload Active Tab",
        subtitle: `Refresh the current web page (${shortcutLabel("reload-tab")})`,
        icon: "🔄",
        badge: "Command",
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
      aria-label="Command Bar"
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
          placeholder="Type URL, search web, switch workspace/tab, or run command..."
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
        <span><kbd>↑</kbd> <kbd>↓</kbd> to navigate</span>
        <span><kbd>↵</kbd> to execute</span>
        <span><kbd>{$keyboardShortcuts && shortcutLabel("command-bar")}</kbd> anytime</span>
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
