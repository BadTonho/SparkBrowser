<script lang="ts">
  import { prepares } from "../../actions/preparePanel";
  import { tick } from "svelte";
  import { get } from "svelte/store";
  import { onMount } from "svelte";
  import TitlebarActionIcon from "./TitlebarActionIcon.svelte";
  import DownloadProgressIndicator from "../downloads/DownloadProgressIndicator.svelte";
  import { t } from "../../i18n";
  import { openTools, isToolsOpen } from "../../stores/toolsStore";
  import { openHistoryDrawer } from "../../stores/persistenceStore";
  import { openDownloadsDrawer, downloadProgressText, isDownloadsDrawerOpen } from "../../stores/downloadsStore";
  import { openCommandBar } from "../../stores/commandBarStore";
  import { tabLayout, showBookmarksBar, toggleBookmarksBar, toggleZenMode, whatsappDisplayMode } from "../../stores/settingsStore";
  import { isWhatsAppWindowOpen, activeWhatsAppDisplayMode } from "../../stores/whatsappStore";
  import { isSettingsQuickOpen, openSettingsQuickMenu } from "../../stores/settingsQuickPopoverStore";
  import { privacyStats } from "../../stores/privacyStore";
  import { openPrivacy, privacyPopover } from "../../stores/privacyPopoverStore";
  import { keyboardShortcuts, shortcutLabel } from "../../stores/shortcutsStore";
  import { toggleExtensionsModal, isExtensionsModalOpen } from "../../stores/extensionsStore";
  import { titlebarActionOrder, shownTitlebarActions, moveTitlebarAction, type TitlebarAction } from "../../stores/titlebarActionsStore";
  import { createActionDrag, type ActionDragPreview } from "../../services/titlebar-action-drag";
  import { isQrCodeOpen, openQrCode } from "../../stores/qrCodeStore";
  import { isResourcesOpen, openResourcesPanel } from "../../stores/resourcesPopoverStore";
import { invoke } from "../../services/tauri-bridge";

  let root = $state<HTMLDivElement>(null!);
  let preview = $state<ActionDragPreview | null>(null);
  let drag: ReturnType<typeof createActionDrag> | undefined;
  // The standalone WhatsApp window ships on Windows first; elsewhere the button stays hidden.
  const supportsWhatsApp = typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");
  const openWhatsApp = async (button: HTMLButtonElement) => {
    const rect = button.getBoundingClientRect();
    const anchor = { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
    if (get(isWhatsAppWindowOpen)) {
      try {
        await invoke("toggle_whatsapp_popover", { anchor, mode: get(activeWhatsAppDisplayMode) });
        isWhatsAppWindowOpen.set(false);
      } catch (error) {
        console.error("WhatsApp window failed", error);
      }
      return;
    }

    const mode = get(whatsappDisplayMode);
    activeWhatsAppDisplayMode.set(mode);
    isWhatsAppWindowOpen.set(true);
    await tick();
    try {
      await invoke("toggle_whatsapp_popover", { anchor, mode });
    } catch (error) {
      isWhatsAppWindowOpen.set(false);
      console.error("WhatsApp window failed", error);
    }
  };
  const visible = $derived($titlebarActionOrder.filter(id => $shownTitlebarActions.includes(id) && (id !== "bookmarks" || $tabLayout === "horizontal") && (id !== "whatsapp" || supportsWhatsApp)));
  $effect(() => { $tabLayout; drag?.cancel(); });

  onMount(() => {
    let pointerTarget: HTMLButtonElement | null = null;
    drag = createActionDrag({
      items: () => [...root.querySelectorAll<HTMLButtonElement>("[data-action]")].map(button => ({
        id: button.dataset.action as TitlebarAction, ...rect(button.getBoundingClientRect()),
      })),
      bounds: () => root.getBoundingClientRect(),
      capture: id => pointerTarget?.setPointerCapture(id),
      release: id => { if (pointerTarget?.hasPointerCapture(id)) pointerTarget.releasePointerCapture(id); },
      preview: value => { preview = value; },
      commit: moveTitlebarAction,
    });
    const down = (event: PointerEvent) => {
      const button = event.target instanceof Element ? event.target.closest<HTMLButtonElement>("[data-action]") : null;
      if (!button || event.button !== 0 || !event.isPrimary) return;
      // Prevent Tauri's ancestor drag region from treating the gesture as a window drag.
      event.stopPropagation();
      pointerTarget = button;
      drag!.down(button.dataset.action as TitlebarAction, event);
    };
    const move = (event: PointerEvent) => drag!.move(event);
    const mouseDown = (event: MouseEvent) => { if (event.button === 0) event.stopPropagation(); };
    const up = (event: PointerEvent) => drag!.up(event);
    const cancel = () => drag!.cancel();
    const click = (event: MouseEvent) => {
      if (drag!.consumeClick(event)) { event.preventDefault(); event.stopImmediatePropagation(); }
    };
    const keys = (event: KeyboardEvent) => {
      if (event.key === "Escape" && drag!.isTracking()) { event.preventDefault(); event.stopImmediatePropagation(); cancel(); }
    };
    root.addEventListener("pointerdown", down);
    root.addEventListener("mousedown", mouseDown);
    root.addEventListener("pointermove", move);
    root.addEventListener("pointerup", up);
    root.addEventListener("pointercancel", cancel);
    root.addEventListener("lostpointercapture", cancel);
    root.addEventListener("click", click, true);
    window.addEventListener("keydown", keys, true);
    window.addEventListener("blur", cancel);
    window.addEventListener("resize", cancel);
    return () => {
      cancel();
      root.removeEventListener("pointerdown", down);
      root.removeEventListener("mousedown", mouseDown);
      root.removeEventListener("pointermove", move);
      root.removeEventListener("pointerup", up);
      root.removeEventListener("pointercancel", cancel);
      root.removeEventListener("lostpointercapture", cancel);
      root.removeEventListener("click", click, true);
      window.removeEventListener("keydown", keys, true);
      window.removeEventListener("blur", cancel);
      window.removeEventListener("resize", cancel);
      drag = undefined;
    };
  });
  function rect(value: DOMRect) {
    return { left: value.left, right: value.right, top: value.top, bottom: value.bottom };
  }
</script>

<div class="titlebar-actions" class:reordering={Boolean(preview)} bind:this={root} role="group" aria-label={$t("titlebar.quick_actions")}>
  <span id="titlebar-reorder-hint" class="sr-only">{$t("titlebar.reorder_hint")}</span>
  <span id="titlebar-download-progress" class="sr-only">{$downloadProgressText}</span>
  {#each visible as id (id)}
    {#if id === "tools"}
      <button
        data-action="tools"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "tools"}
        style:translate={preview?.id === "tools" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        use:prepares={"tools"}
        onclick={event => openTools("sessions", event.currentTarget)}
        aria-haspopup="dialog" aria-expanded={$isToolsOpen}
        title={$t("titlebar.tools_ea93") + " · " + $t("titlebar.reorder_hint")}
        data-testid="open-tools" aria-label={$t("titlebar.tools_ea93")}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "bookmarks"}
      <button
        data-action="bookmarks"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "bookmarks"}
        style:translate={preview?.id === "bookmarks" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        class:active={$showBookmarksBar}
        onclick={toggleBookmarksBar}
        title={($showBookmarksBar ? $t("titlebar.hide_bookmarks_bar_31b5") : $t("titlebar.show_bookmarks_bar_570b")) + " · " + $t("titlebar.reorder_hint")}
        aria-label={$showBookmarksBar ? $t("titlebar.hide_bookmarks_bar_31b5") : $t("titlebar.show_bookmarks_bar_570b")}
        aria-pressed={$showBookmarksBar}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "commands"}
      <button
        data-action="commands"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "commands"}
        style:translate={preview?.id === "commands" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        onclick={() => openCommandBar()}
        title={($t("titlebar.open_command_bar_p0_cf83", { p0: $keyboardShortcuts && shortcutLabel("command-bar") })) + " · " + $t("titlebar.reorder_hint")}
        aria-label={$t("titlebar.command_bar_1356")}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "history"}
      <button
        data-action="history"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "history"}
        style:translate={preview?.id === "history" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        use:prepares={"history"}
        onclick={openHistoryDrawer}
        title={($t("titlebar.browsing_history_p0_cf44", { p0: $keyboardShortcuts && shortcutLabel("history") })) + " · " + $t("titlebar.reorder_hint")}
        data-testid="open-history" aria-label={$t("titlebar.browsing_history_13cc")}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "downloads"}
      <button
        data-action="downloads"
        aria-describedby="titlebar-reorder-hint titlebar-download-progress"
        class:dragging={preview?.id === "downloads"}
        style:translate={preview?.id === "downloads" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        use:prepares={"downloads"}
        onclick={event => openDownloadsDrawer(event.currentTarget)}
        aria-haspopup="dialog" aria-expanded={$isDownloadsDrawerOpen}
        title={($t("titlebar.download_manager_p0_8620", { p0: $keyboardShortcuts && shortcutLabel("downloads") })) + ($downloadProgressText ? " · " + $downloadProgressText : "") + " · " + $t("titlebar.reorder_hint")}
        data-testid="open-downloads" aria-label={$t("titlebar.downloads_d5fd")}
      >
        <TitlebarActionIcon action={id} />
        <DownloadProgressIndicator />
      </button>
    {/if}
    {#if id === "privacy"}
      <button
        data-action="privacy"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "privacy"}
        style:translate={preview?.id === "privacy" ? `${preview.offset}px 0` : undefined}
        class="icon-btn shield-btn"
        class:shield-on={$privacyStats.shield_enabled}
        use:prepares={"privacy"}
        onclick={event => openPrivacy(event.currentTarget)}
        title={($t("privacy.controls")) + " · " + $t("titlebar.reorder_hint")}
        aria-label={$t("privacy.controls")}
        aria-haspopup="dialog" aria-expanded={Boolean($privacyPopover)} data-testid="open-privacy"
      >
        <TitlebarActionIcon action={id} />
        <span class="shield-dot"></span>
      </button>
    {/if}
    {#if id === "extensions"}
      <button
        data-action="extensions"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "extensions"}
        style:translate={preview?.id === "extensions" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        class:active={$isExtensionsModalOpen}
        onclick={() => void toggleExtensionsModal()}
        title={$t("extensions.manage") + " · " + $t("titlebar.reorder_hint")}
        data-testid="open-extensions" aria-label={$t("extensions.title")} aria-haspopup="dialog" aria-expanded={$isExtensionsModalOpen}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "settings"}
      <button
        data-action="settings"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "settings"}
        style:translate={preview?.id === "settings" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        use:prepares={["quick-settings", "settings"]}
        onclick={event => openSettingsQuickMenu(event.currentTarget)}
        title={($t("titlebar.settings_shortcuts_p0_82b9", { p0: $keyboardShortcuts && shortcutLabel("settings") })) + " · " + $t("titlebar.reorder_hint")}
        data-testid="open-settings" aria-label={$t("titlebar.settings_74a8")} aria-haspopup="dialog" aria-expanded={$isSettingsQuickOpen}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "zen"}
      <button
        data-action="zen"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "zen"}
        style:translate={preview?.id === "zen" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        onclick={toggleZenMode}
        title={($t("titlebar.enter_zen_focus_mode_p0_d689", { p0: $keyboardShortcuts && shortcutLabel("zen") })) + " · " + $t("titlebar.reorder_hint")}
        aria-label={$t("titlebar.zen_mode_826b")}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "qr"}
      <button
        data-action="qr"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "qr"}
        style:translate={preview?.id === "qr" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        onclick={openQrCode}
        title={$t("qr_code.title") + " · " + $t("titlebar.reorder_hint")}
        aria-label={$t("qr_code.title")}
        aria-haspopup="dialog" aria-expanded={$isQrCodeOpen}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "resources"}
      <button
        data-action="resources"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "resources"}
        style:translate={preview?.id === "resources" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        class:active={$isResourcesOpen}
        use:prepares={"resources"}
        onclick={event => openResourcesPanel(event.currentTarget)}
        title={$t("resources.title") + " · " + $t("titlebar.reorder_hint")}
        data-testid="open-resources" aria-label={$t("resources.title")} aria-haspopup="dialog" aria-expanded={$isResourcesOpen}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
    {#if id === "whatsapp"}
      <button
        data-action="whatsapp"
        aria-describedby="titlebar-reorder-hint"
        class:dragging={preview?.id === "whatsapp"}
        style:translate={preview?.id === "whatsapp" ? `${preview.offset}px 0` : undefined}
        class="icon-btn"
        onclick={event => openWhatsApp(event.currentTarget)}
        title={$t("whatsapp.open") + " · " + $t("titlebar.reorder_hint")}
        aria-label={$t("whatsapp.open")}
      >
        <TitlebarActionIcon action={id} />
      </button>
    {/if}
  {/each}
  {#if preview?.marker != null}
    <span class="insertion-marker" aria-hidden="true" style:left="{preview.marker - root.getBoundingClientRect().left - 2}px"></span>
  {/if}
  <span class="sr-only" role="status">{preview ? $t("titlebar.reordering") : ""}</span>
</div>

<style>
  .titlebar-actions { position: relative; display: flex; align-items: center; gap: var(--titlebar-control-gap, 3px); height: var(--titlebar-actions-height, 32px); touch-action: none; user-select: none; }
  .sr-only { position: absolute; width: 1px; height: 1px; padding: 0; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  .insertion-marker { position: absolute; top: 2px; bottom: 2px; width: 2px; border-radius: 2px; background: var(--accent-primary); pointer-events: none; }
  .reordering .icon-btn { cursor: grabbing; }
  .icon-btn.dragging { z-index: 1; opacity: .75; background: var(--bg-glass-hover); border-color: var(--accent-primary); box-shadow: 0 2px 8px #0006; pointer-events: none; }
  .icon-btn:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 1px; }
  .icon-btn {
    position: relative;
    width: var(--titlebar-control-width, 27px);
    height: var(--titlebar-control-height, 26px);
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--titlebar-control-font-size, 12px);
    color: var(--text-secondary);
    border: 1px solid transparent;
    flex-shrink: 0;
    transition:
      background 0.18s ease,
      color 0.18s ease,
      border-color 0.18s ease,
      transform 0.15s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .icon-btn:hover {
    background: var(--bg-glass-hover);
    color: var(--text-primary);
    transform: scale(1.05);
  }

  .icon-btn.active {
    color: var(--accent-primary);
    background: color-mix(in srgb, var(--accent-primary) 16%, transparent);
  }

  .shield-btn {
    opacity: 0.65;
  }

  .shield-btn.shield-on {
    opacity: 1;
    background: var(--status-on-bg);
    border-color: var(--status-on-border);
  }

  .shield-dot {
    position: absolute;
    bottom: 2px;
    right: 2px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--text-muted);
    transition: background 0.2s ease, box-shadow 0.2s ease;
  }

  .shield-btn.shield-on .shield-dot {
    background: var(--status-on);
    box-shadow: 0 0 6px var(--status-on);
  }

</style>
