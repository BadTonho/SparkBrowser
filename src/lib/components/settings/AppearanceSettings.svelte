<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import LanguageSettings from "./LanguageSettings.svelte";
  import PipAppearanceSettings from "./PipAppearanceSettings.svelte";
  import { t } from "../../i18n";
import {
  isZenMode,
  toggleZenMode,
  overlayBlurPx,
  setOverlayBlurPx,
  tabLayout,
  setTabLayout,
  showBookmarksBar,
  setShowBookmarksBar,
  workspaceSwitcherDisplay,
  setWorkspaceSwitcherDisplay,
  themeMode,
  setThemeMode,
  windowSurfaceStyle,
  setWindowSurfaceStyle,
  windowSurfaceOpacity,
  setWindowSurfaceOpacity,
  windowSurfaceBlurPx,
  setWindowSurfaceBlurPx,
  wideAddressBar,
  setWideAddressBar,
  showFullUrls,
  setShowFullUrls,
  compactToolbar,
  setCompactToolbar,
  roundedContentFrame,
  setRoundedContentFrame,
  whatsappDisplayMode,
  setWhatsAppDisplayMode,
  MIN_WINDOW_SURFACE_OPACITY,
  MAX_WINDOW_SURFACE_OPACITY,
  MAX_WINDOW_SURFACE_BLUR_PX,
} from "../../stores/settingsStore";
import { formatShortcut, getShortcutBinding, keyboardShortcuts } from "../../stores/shortcutsStore";

const supportsWhatsApp = typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");
</script>

<section class="setting-group" aria-labelledby="appearance-heading">
  <h2 class="group-title" id="appearance-heading"><AppIcon name="monitor" /> {$t("settings.appearance_layout_a2b1")}</h2>

  <LanguageSettings />
  <PipAppearanceSettings />

  <div class="setting-row" id="setting-theme-mode" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.theme_mode")}</span>
      <span class="row-desc">{$t("settings.theme_mode_description")}</span>
    </div>
    <div class="timer-pills">
      <button
        class="mini-pill"
        class:selected={$themeMode === "dark"}
        aria-pressed={$themeMode === "dark"}
        onclick={() => setThemeMode("dark")}
      >
        <AppIcon name="moon" /> {$t("settings.theme_dark")}
      </button>
      <button
        class="mini-pill"
        class:selected={$themeMode === "light"}
        aria-pressed={$themeMode === "light"}
        onclick={() => setThemeMode("light")}
      >
        <AppIcon name="sun" /> {$t("settings.theme_light")}
      </button>
    </div>
  </div>

  <div class="setting-row" id="setting-browser-layout" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.browser_layout_28d1")}</span>
      <span class="row-desc">{$t("settings.choose_a_horizontal_browser_layout_or_a_vertical_sidebar_layout_a4a5")}</span>
    </div>
    <div class="timer-pills">
      <button
        class="mini-pill"
        class:selected={$tabLayout === "horizontal"}
        onclick={() => setTabLayout("horizontal")}
        title={$t("settings.tabs_at_the_top_workspaces_beside_search_and_bookmarks_below_e1af")}
      >
        {$t("settings.horizontal_9b55")}
      </button>
      <button
        class="mini-pill"
        class:selected={$tabLayout === "vertical"}
        onclick={() => setTabLayout("vertical")}
        title={$t("settings.workspaces_bookmarks_and_tabs_in_the_sidebar_5cff")}
      >
        {$t("settings.vertical_b6d6")}
      </button>
    </div>
  </div>

  {#if supportsWhatsApp}
    <div class="setting-row" id="setting-whatsapp-display-mode" tabindex="-1">
      <div class="row-text">
        <span class="row-label">{$t("settings.whatsapp_display_mode")}</span>
        <span class="row-desc">{$t("settings.whatsapp_display_mode_description")}</span>
      </div>
      <div class="timer-pills">
        <button
          class="mini-pill"
          class:selected={$whatsappDisplayMode === "popover"}
          aria-pressed={$whatsappDisplayMode === "popover"}
          onclick={() => setWhatsAppDisplayMode("popover")}
        >
          {$t("settings.whatsapp_popover")}
        </button>
        <button
          class="mini-pill"
          class:selected={$whatsappDisplayMode === "side_panel"}
          aria-pressed={$whatsappDisplayMode === "side_panel"}
          onclick={() => setWhatsAppDisplayMode("side_panel")}
        >
          {$t("settings.whatsapp_side_panel")}
        </button>
      </div>
    </div>
  {/if}

  <h3 class="group-title visual-settings-heading">{$t("settings.toolbar_appearance")}</h3>

  <label class="visual-toggle-row" id="setting-wide-address-bar" tabindex="-1">
    <span class="row-text">
      <span class="row-label">{$t("settings.wide_address_bar")}</span>
      <span class="row-desc">{$t("settings.wide_address_bar_description")}</span>
    </span>
    <span class="visual-switch">
      <input type="checkbox" role="switch" checked={$wideAddressBar} onchange={event => setWideAddressBar((event.currentTarget as HTMLInputElement).checked)} />
      <span class="visual-switch-track" aria-hidden="true"></span>
    </span>
  </label>

  <label class="visual-toggle-row" id="setting-full-urls" tabindex="-1">
    <span class="row-text">
      <span class="row-label">{$t("settings.show_full_urls")}</span>
      <span class="row-desc">{$t("settings.show_full_urls_description")}</span>
    </span>
    <span class="visual-switch">
      <input type="checkbox" role="switch" checked={$showFullUrls} onchange={event => setShowFullUrls((event.currentTarget as HTMLInputElement).checked)} />
      <span class="visual-switch-track" aria-hidden="true"></span>
    </span>
  </label>

  <label class="visual-toggle-row" id="setting-compact-toolbar" tabindex="-1">
    <span class="row-text">
      <span class="row-label">{$t("settings.compact_toolbar")}</span>
      <span class="row-desc">{$t("settings.compact_toolbar_description")}</span>
    </span>
    <span class="visual-switch">
      <input type="checkbox" role="switch" checked={$compactToolbar} onchange={event => setCompactToolbar((event.currentTarget as HTMLInputElement).checked)} />
      <span class="visual-switch-track" aria-hidden="true"></span>
    </span>
  </label>

  <label class="visual-toggle-row" id="setting-rounded-content-frame" tabindex="-1">
    <span class="row-text">
      <span class="row-label">{$t("settings.rounded_content_frame")}</span>
      <span class="row-desc">{$t("settings.rounded_content_frame_description")}</span>
    </span>
    <span class="visual-switch">
      <input type="checkbox" role="switch" checked={$roundedContentFrame} onchange={event => setRoundedContentFrame((event.currentTarget as HTMLInputElement).checked)} />
      <span class="visual-switch-track" aria-hidden="true"></span>
    </span>
  </label>

  <div class="setting-row" id="setting-window-surface-style" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.window_surface_style")}</span>
      <span class="row-desc">{$t("settings.window_surface_style_desc")}</span>
    </div>
    <div class="timer-pills">
      <button
        class="mini-pill"
        class:selected={$windowSurfaceStyle === "frosted"}
        aria-pressed={$windowSurfaceStyle === "frosted"}
        onclick={() => setWindowSurfaceStyle("frosted")}
      >
        <AppIcon name="window" /> {$t("settings.window_surface_frosted")}
      </button>
      <button
        class="mini-pill"
        class:selected={$windowSurfaceStyle === "solid"}
        aria-pressed={$windowSurfaceStyle === "solid"}
        onclick={() => setWindowSurfaceStyle("solid")}
      >
        <AppIcon name="square-filled" /> {$t("settings.window_surface_solid")}
      </button>
    </div>
  </div>

  {#if $windowSurfaceStyle === "frosted"}
    <div class="setting-row" id="setting-window-surface-opacity" tabindex="-1">
      <div class="row-text">
        <span class="row-label">{$t("settings.window_surface_opacity")}</span>
        <span class="row-desc">{$t("settings.window_surface_opacity_desc")}</span>
      </div>
      <div class="range-control">
        <input
          type="range"
          min={MIN_WINDOW_SURFACE_OPACITY}
          max={MAX_WINDOW_SURFACE_OPACITY}
          step="1"
          value={$windowSurfaceOpacity}
          oninput={(event) => setWindowSurfaceOpacity(Number(event.currentTarget.value))}
          aria-label={$t("settings.window_surface_opacity")}
          aria-valuetext="{$windowSurfaceOpacity}%"
        />
        <output>{ $windowSurfaceOpacity }%</output>
      </div>
    </div>

    <div class="setting-row" id="setting-window-surface-blur" tabindex="-1">
      <div class="row-text">
        <span class="row-label">{$t("settings.window_surface_blur")}</span>
        <span class="row-desc">{$t("settings.window_surface_blur_desc")}</span>
      </div>
      <div class="range-control">
        <input
          type="range"
          min="0"
          max={MAX_WINDOW_SURFACE_BLUR_PX}
          step="1"
          value={$windowSurfaceBlurPx}
          oninput={(event) => setWindowSurfaceBlurPx(Number(event.currentTarget.value))}
          aria-label={$t("settings.window_surface_blur")}
          aria-valuetext="{$windowSurfaceBlurPx} px"
        />
        <output>{ $windowSurfaceBlurPx } px</output>
      </div>
    </div>
  {/if}

  <div class="setting-row" id="setting-background-blur" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.background_blur_c21c")}</span>
      <span class="row-desc">{$t("settings.adjust_blur_behind_browser_overlays_4f2a")}</span>
    </div>
    <div class="range-control">
      <input
        type="range"
        min="0"
        max="24"
        step="1"
        value={$overlayBlurPx}
        oninput={(event) => setOverlayBlurPx(Number(event.currentTarget.value))}
        aria-label={$t("settings.background_blur_intensity_9873")}
        aria-valuetext={$t("settings.p0_pixels_b580", { p0: $overlayBlurPx })}
      />
      <output>{ $overlayBlurPx } px</output>
    </div>
  </div>

  <div class="setting-row" id="setting-bookmarks-bar" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.bookmarks_bar_d670")}</span>
      <span class="row-desc">{$t("settings.show_or_hide_the_horizontal_bookmarks_bar_without_removing_saved_bookmar_85a7")}</span>
    </div>
    <button
      class="pill-action"
      class:active={$showBookmarksBar}
      aria-pressed={$showBookmarksBar}
      onclick={() => setShowBookmarksBar(!$showBookmarksBar)}
    >
      {$showBookmarksBar ? $t("settings.visible_8411") : $t("settings.hidden_7e6f")}
    </button>
  </div>

  <div class="setting-row" id="setting-workspace-switcher-display" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.workspace_switcher_43f1")}</span>
      <span class="row-desc">{$t("settings.choose_horizontal_buttons_or_a_floating_picker_vertical_sidebar_buttons__8411")}</span>
    </div>
    <div class="timer-pills">
      <button
        class="mini-pill"
        class:selected={$workspaceSwitcherDisplay === "toolbar"}
        aria-pressed={$workspaceSwitcherDisplay === "toolbar"}
        onclick={() => setWorkspaceSwitcherDisplay("toolbar")}
      >
        {$t("settings.toolbar_buttons_1962")}
      </button>
      <button
        class="mini-pill"
        class:selected={$workspaceSwitcherDisplay === "popup"}
        aria-pressed={$workspaceSwitcherDisplay === "popup"}
        onclick={() => setWorkspaceSwitcherDisplay("popup")}
      >
        {$t("settings.popup_4c14")}
      </button>
    </div>
  </div>

  <div class="setting-row" id="setting-zen-mode" tabindex="-1">
    <div class="row-text">
      <span class="row-label">{$t("settings.zen_focus_mode_092d")}{$keyboardShortcuts && formatShortcut(getShortcutBinding($keyboardShortcuts, "zen")!)})</span>
      <span class="row-desc">{$t("settings.hides_all_browser_chrome_so_websites_use_100_of_the_viewport_3078")}</span>
    </div>
    <button
      class="pill-action"
      class:active={$isZenMode}
      onclick={toggleZenMode}
    >
      {$isZenMode ? $t("settings.active_9234") : $t("settings.off_ca79")}
    </button>
  </div>
</section>

<style>
  .visual-settings-heading { margin: 10px 0 0; }
  .visual-toggle-row {
    min-height: 58px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-subtle);
    cursor: pointer;
  }
  .visual-toggle-row:first-of-type { border-top: 1px solid var(--border-subtle); }
  .visual-toggle-row:focus-visible { outline: 2px solid var(--accent-primary); outline-offset: 2px; border-radius: var(--radius-sm); }
  .visual-toggle-row .row-text { gap: 4px; }
  .visual-switch { position: relative; flex: 0 0 auto; width: 40px; height: 22px; }
  .visual-switch input { position: absolute; inset: 0; z-index: 1; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: pointer; }
  .visual-switch-track { position: absolute; inset: 0; border-radius: 99px; background: rgba(255,255,255,.2); transition: background .16s ease; }
  .visual-switch-track::before { content: ""; position: absolute; top: 3px; left: 3px; width: 16px; height: 16px; border-radius: 50%; background: #fff; transition: transform .16s ease; }
  .visual-switch input:checked + .visual-switch-track { background: var(--accent-primary); }
  .visual-switch input:checked + .visual-switch-track::before { transform: translateX(18px); }
  .visual-switch input:focus-visible + .visual-switch-track { outline: 2px solid var(--accent-primary); outline-offset: 3px; }
  @media (max-width: 720px) { .visual-toggle-row { padding-inline: 4px; } }
  @media (prefers-reduced-motion: reduce) { .visual-switch-track, .visual-switch-track::before { transition: none; } }
</style>
