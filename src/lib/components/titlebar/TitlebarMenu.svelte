<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { get } from "svelte/store";
  import { t } from "../../i18n";
  import { titlebarMenu, closeTitlebarMenu } from "../../stores/titlebarMenuStore";
  import { onOverlayOpening } from "../../services/bridge/overlays";
  import { openTools } from "../../stores/toolsStore";
  import { openHistoryDrawer } from "../../stores/persistenceStore";
  import { openDownloadsDrawer } from "../../stores/downloadsStore";
  import { openCommandBar } from "../../stores/commandBarStore";
  import { showBookmarksBar, toggleBookmarksBar, openSettingsModal, toggleZenMode } from "../../stores/settingsStore";
  import { openPrivacy } from "../../stores/privacyPopoverStore";

  let menu: HTMLDivElement | undefined = $state();
  let position = $state({ top: 40, right: 8 });
  let busy = $state(false);

  function measure() {
    const anchor = get(titlebarMenu)?.anchor.getBoundingClientRect();
    if (anchor) position = { top: anchor.bottom + 6, right: Math.max(8, window.innerWidth - anchor.right) };
  }

  async function run(action: "tools" | "bookmarks" | "commands" | "history" | "downloads" | "privacy" | "settings" | "zen") {
    const session = get(titlebarMenu);
    if (!session || busy) return;
    busy = true;
    // Dialogs remember this stable button rather than a menu item removed during handoff.
    session.anchor.focus({ preventScroll: true });
    try {
      switch (action) {
        case "tools": await openTools(); break;
        case "bookmarks": toggleBookmarksBar(); break;
        case "commands": await openCommandBar(); break;
        case "history": await openHistoryDrawer(); break;
        case "downloads": await openDownloadsDrawer(); break;
        case "settings": await openSettingsModal(); break;
        case "zen": toggleZenMode(); break;
        case "privacy":
          // The native privacy popover requires ordinary page visibility and a persistent anchor.
          await closeTitlebarMenu();
          await openPrivacy(session.anchor);
          break;
      }
    } finally {
      await closeTitlebarMenu(action === "bookmarks");
      busy = false;
    }
  }

  function keys(event: KeyboardEvent) {
    const items = [...(menu?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
    const index = items.indexOf(document.activeElement as HTMLButtonElement);
    let next: number;
    switch (event.key) {
      case "ArrowDown": next = (index + 1) % items.length; break;
      case "ArrowUp": next = (index - 1 + items.length) % items.length; break;
      case "Home": next = 0; break;
      case "End": next = items.length - 1; break;
      default: return;
    }
    event.preventDefault();
    event.stopPropagation();
    items[next]?.focus();
  }

  $effect(() => {
    const session = $titlebarMenu;
    if (!session) return;
    untrack(measure);
    void tick().then(() => {
      if (get(titlebarMenu) === session) menu?.querySelector<HTMLButtonElement>("button")?.focus();
    });
  });

  onMount(() => {
    const stop = onOverlayOpening(id => { if (id !== "titlebar-menu") void closeTitlebarMenu(); });
    const dismiss = (event: KeyboardEvent) => {
      if (!get(titlebarMenu) || !["Escape", "Tab"].includes(event.key)) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      void closeTitlebarMenu(true);
    };
    window.addEventListener("keydown", dismiss, true);
    return () => {
      stop();
      window.removeEventListener("keydown", dismiss, true);
      void closeTitlebarMenu();
    };
  });
</script>

<svelte:window onresize={measure} onblur={() => closeTitlebarMenu()} />

{#if $titlebarMenu}
  <div class="menu-dismiss" role="presentation" onclick={() => closeTitlebarMenu(true)}>
    <div class="titlebar-menu" data-testid="titlebar-menu" role="menu" aria-label={$t("titlebar.more")} tabindex="-1" bind:this={menu}
      style:top="{position.top}px" style:right="{position.right}px" onclick={event => event.stopPropagation()} onkeydown={keys}>
      <button role="menuitem" data-testid="open-tools" disabled={busy} onclick={() => run("tools")}>{$t("titlebar.tools_ea93")}</button>
      <button role="menuitemcheckbox" data-testid="toggle-bookmarks" aria-checked={$showBookmarksBar} disabled={busy} onclick={() => run("bookmarks")}>{$showBookmarksBar ? $t("titlebar.hide_bookmarks_bar_31b5") : $t("titlebar.show_bookmarks_bar_570b")}</button>
      <button role="menuitem" data-testid="open-command-bar" disabled={busy} onclick={() => run("commands")}>{$t("titlebar.command_bar_1356")}</button>
      <button role="menuitem" data-testid="open-history" disabled={busy} onclick={() => run("history")}>{$t("titlebar.browsing_history_13cc")}</button>
      <button role="menuitem" data-testid="open-downloads" disabled={busy} onclick={() => run("downloads")}>{$t("titlebar.downloads_d5fd")}</button>
      <button role="menuitem" data-testid="open-privacy" disabled={busy} onclick={() => run("privacy")}>{$t("privacy.controls")}</button>
      <button role="menuitem" data-testid="open-settings" disabled={busy} onclick={() => run("settings")}>{$t("titlebar.settings_74a8")}</button>
      <button role="menuitem" data-testid="toggle-zen" disabled={busy} onclick={() => run("zen")}>{$t("titlebar.zen_mode_826b")}</button>
    </div>
  </div>
{/if}

<style>
  .menu-dismiss { position:fixed; inset:0; z-index:1000; }
  .titlebar-menu { position:absolute; display:flex; flex-direction:column; gap:2px; width:240px; max-width:calc(100vw - 16px); max-height:calc(100vh - 56px); overflow-y:auto; padding:6px; background:var(--bg-primary,#1c1c1c); border:1px solid var(--border-subtle); border-radius:var(--radius-md); box-shadow:0 8px 24px #0006; }
  button { padding:8px 12px; border-radius:var(--radius-sm); text-align:left; color:var(--text-primary); font-size:12px; }
  button:hover,button:focus-visible { background:var(--bg-glass-hover); outline:2px solid var(--accent-primary); outline-offset:-2px; }
  button:disabled { opacity:.5; }
</style>
