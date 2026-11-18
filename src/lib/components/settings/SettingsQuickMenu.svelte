<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { focusDialog } from "../../actions/focusDialog";
  import { panelPosition } from "../../services/panel-presentation";
  import { mountSettingsQuickHost } from "./settings-quick-host";
  import { t, type MessageKey } from "../../i18n";
  import {
    isSettingsQuickOpen,
    settingsQuickSession,
    closeSettingsQuickMenu,
  } from "../../stores/settingsQuickPopoverStore";
  import {
    openSettingsModal,
    setThemeMode,
    themeMode,
    setTabLayout,
    tabLayout,
    setShowBookmarksBar,
    showBookmarksBar,
  } from "../../stores/settingsStore";
  import TitlebarActionIcon from "../titlebar/TitlebarActionIcon.svelte";
  import { DEFAULT_TITLEBAR_ACTIONS, ALWAYS_SHOWN_ACTIONS, shownTitlebarActions, setTitlebarActionShown, type TitlebarAction } from "../../stores/titlebarActionsStore";
  import "./settings-quick.css";

  const native = isTauriRuntime();
  let viewport = $state({ width: innerWidth, height: innerHeight });
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  const position = $derived(panelPosition(anchor, viewport.width, viewport.height, { width: 300, maxHeight: 560 }));

  function host(node: HTMLElement) {
    const stop = mountSettingsQuickHost(node, native);
    const focus = native ? undefined : focusDialog(node, { onEscape: () => closeSettingsQuickMenu(true) });
    let frame = 0;
    const measure = () => {
      const rect = $settingsQuickSession?.anchor.getBoundingClientRect();
      if (rect && (anchor.left !== rect.left || anchor.top !== rect.top || anchor.width !== rect.width || anchor.height !== rect.height)) {
        anchor = { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
      }
      frame = requestAnimationFrame(measure);
    };
    if (!native) measure();
    return { destroy() { cancelAnimationFrame(frame); stop(); focus?.destroy(); } };
  }

  // The standalone WhatsApp window ships on Windows first; the bookmarks button only exists with the horizontal layout.
  const supportsWhatsApp = typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");
  const toolbarLabels: Record<TitlebarAction, MessageKey> = {
    tools: "settings.toolbar_tools", bookmarks: "settings.toolbar_bookmarks", commands: "settings.toolbar_commands",
    history: "settings.toolbar_history", downloads: "settings.toolbar_downloads", privacy: "settings.toolbar_privacy",
    extensions: "settings.toolbar_extensions", settings: "settings.toolbar_settings", zen: "settings.toolbar_zen",
    qr: "settings.toolbar_qr", whatsapp: "settings.toolbar_whatsapp",
  };
  const toolbarButtons = $derived(DEFAULT_TITLEBAR_ACTIONS.filter(id =>
    !ALWAYS_SHOWN_ACTIONS.includes(id) && (id !== "whatsapp" || supportsWhatsApp) && (id !== "bookmarks" || $tabLayout === "horizontal")));

  function openAllSettings() {
    void openSettingsModal();
  }
</script>

<svelte:window onresize={() => { viewport = { width: innerWidth, height: innerHeight }; }} />

{#if $isSettingsQuickOpen}
  <div
    class="settings-quick-dismiss"
    class:settings-quick-controller={native}
    aria-hidden={native ? "true" : undefined}
    role="presentation"
    onclick={(event) => { if (event.target === event.currentTarget) void closeSettingsQuickMenu(false); }}
  >
    <div class="settings-quick-surface" style:left="{position.left}px" style:top="{position.top}px" style:width="{position.width}px" style:max-height="{position.maxHeight}px">
      <div
        class="settings-quick-panel"
        data-testid="settings-quick-menu"
        role="dialog"
        aria-modal="false"
        aria-label={$t("settings.quick_settings")}
        tabindex="-1"
        use:host
      >
        <h2>{$t("settings.quick_settings")}</h2>

        <div class="settings-quick-section" aria-labelledby="settings-quick-theme-label">
          <h3 id="settings-quick-theme-label">{$t("settings.theme_mode")}</h3>
          <div class="settings-quick-options settings-quick-theme-options" role="group" aria-label={$t("settings.theme_mode")}>
            <button class="settings-quick-theme-button" class:active={$themeMode === "light"} aria-pressed={$themeMode === "light"} data-testid="settings-quick-theme-light" onclick={() => setThemeMode("light")}><AppIcon name="sun" /> {$t("settings.theme_light")}</button>
            <button class="settings-quick-theme-button" class:active={$themeMode === "dark"} aria-pressed={$themeMode === "dark"} data-testid="settings-quick-theme-dark" onclick={() => setThemeMode("dark")}><AppIcon name="moon" /> {$t("settings.theme_dark")}</button>
          </div>
        </div>

        <button
          class="settings-quick-toggle"
          aria-pressed={$showBookmarksBar}
          data-testid="settings-quick-bookmarks"
          onclick={() => setShowBookmarksBar(!$showBookmarksBar)}
        >
          <span>{$t("settings.bookmarks_bar_d670")}</span>
          <span class="settings-quick-state">{$showBookmarksBar ? $t("settings.visible_8411") : $t("settings.hidden_7e6f")}</span>
        </button>

        <div class="settings-quick-section" aria-labelledby="settings-quick-layout-label">
          <h3 id="settings-quick-layout-label">{$t("settings.browser_layout_28d1")}</h3>
          <div class="settings-quick-options" role="group" aria-label={$t("settings.browser_layout_28d1")}>
            <button class:active={$tabLayout === "horizontal"} aria-pressed={$tabLayout === "horizontal"} data-testid="settings-quick-layout-horizontal" onclick={() => setTabLayout("horizontal")}>{$t("settings.horizontal_9b55")}</button>
            <button class:active={$tabLayout === "vertical"} aria-pressed={$tabLayout === "vertical"} data-testid="settings-quick-layout-vertical" onclick={() => setTabLayout("vertical")}>{$t("settings.vertical_b6d6")}</button>
          </div>
        </div>

        <div class="settings-quick-section" aria-labelledby="settings-quick-toolbar-label">
          <h3 id="settings-quick-toolbar-label">{$t("settings.toolbar_buttons")}</h3>
          <div class="settings-quick-options settings-quick-toolbar" role="group" aria-label={$t("settings.toolbar_buttons")}>
            {#each toolbarButtons as id (id)}
              <button
                class="settings-quick-toolbar-button"
                class:active={$shownTitlebarActions.includes(id)}
                aria-pressed={$shownTitlebarActions.includes(id)}
                data-testid="settings-quick-toolbar-{id}"
                onclick={() => setTitlebarActionShown(id, !$shownTitlebarActions.includes(id))}
              ><TitlebarActionIcon action={id} /><span>{$t(toolbarLabels[id])}</span></button>
            {/each}
          </div>
        </div>

        <button class="settings-quick-all" data-testid="settings-quick-all" onclick={openAllSettings}>
          {$t("settings.open_all_settings")}
          <span aria-hidden="true">›</span>
        </button>
      </div>
    </div>
  </div>
{/if}
